// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{NavLink, data::*, game_state::*, reasoning};
use dioxus::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn save(mut state: Signal<Play>) {
    let result = save_history(&state.peek().history);
    if let Err(e) = result {
        state.write().storage = e;
    }
}
fn boot(mut state: Signal<Play>, mut cancellation: Signal<Arc<AtomicBool>>) {
    reasoning::cancel(&cancellation.peek());
    let flag = Arc::new(AtomicBool::new(false));
    cancellation.set(flag.clone());
    {
        let mut s = state.write();
        s.phase = Phase::Loading;
        s.message = "Starting the local engine and loading the full constitution…".into();
    }
    spawn(async move {
        let result = reasoning::initialize(flag.clone()).await;
        if flag.load(Ordering::Relaxed) {
            return;
        }
        let mut s = state.write();
        match result {
            Ok(()) => {
                s.loaded = true;
                s.phase = Phase::Ready;
                s.message = "Engine ready. Choose a move to execute its complete record.".into();
            }
            Err(e) => {
                s.phase = Phase::Failed;
                s.message = format!("Engine startup failed: {e}");
            }
        }
    });
}
fn select(mut state: Signal<Play>, flag: Signal<Arc<AtomicBool>>, id: &str) {
    if state.peek().loaded {
        reasoning::cancel(&flag.peek());
    }
    state.write().select(id);
}
#[cfg(not(feature = "desktop"))]
fn location_fragment() -> String {
    #[cfg(feature = "web")]
    {
        web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .map(|h| h.trim_start_matches('#').to_string())
            .unwrap_or_default()
    }
    #[cfg(not(feature = "web"))]
    {
        String::new()
    }
}
fn open_link(mut state: Signal<Play>, flag: Signal<Arc<AtomicBool>>, fragment: &str) {
    let person = if let Some(id) = fragment.strip_prefix("fork=") {
        game()
            .scenarios
            .iter()
            .find(|f| f.id == id)
            .map(|f| (f.id.clone(), Some(f.person.clone())))
    } else if let Some(id) = fragment.strip_prefix("joint=") {
        game()
            .joints
            .iter()
            .find(|j| j.id == id)
            .map(|j| (j.id.clone(), None))
    } else {
        None
    };
    let Some((id, person)) = person else {
        return;
    };
    if state.peek().selection == id {
        return;
    }
    select(state, flag, &id);
    if let Some(person) = person {
        state.write().person = person;
    }
    let _ = document::eval(
        "setTimeout(()=>{const e=document.querySelector('.play-card');if(e){e.scrollIntoView({behavior:'instant'});}},50);",
    );
}
fn queue(mut state: Signal<Play>, flag: Signal<Arc<AtomicBool>>, id: String) {
    reasoning::cancel(&flag.peek());
    let mut s = state.write();
    s.request += 1;
    s.active = false;
    s.pending = Some(Job {
        id,
        background: false,
        owner: s.selection.clone(),
    });
}
#[component]
pub fn Game() -> Element {
    let mut state = use_signal(Play::default);
    #[cfg(feature = "desktop")]
    {
        let session = use_context::<crate::Session>();
        use_effect(move || {
            let fragment = (session.fragment)();
            let eval = document::eval(
                "const id=await dioxus.recv();setTimeout(()=>{const e=document.getElementById(id);if(e){e.scrollIntoView({behavior:'instant'});e.setAttribute('tabindex','-1');e.focus({preventScroll:true})}},50);",
            );
            let _ = eval.send(fragment);
        });
    }
    let mut cancellation = use_signal(|| Arc::new(AtomicBool::new(false)));
    let mut share = use_signal(String::new);
    let mut copy_label = use_signal(|| "Share progress".to_string());
    use_effect(move || {
        match load_history() {
            Ok(history) => {
                let mut s = state.write();
                s.restore = history
                    .completed
                    .iter()
                    .chain(&history.measured)
                    .cloned()
                    .collect();
                s.history = history;
            }
            Err(e) => state.write().storage = e,
        }
        boot(state, cancellation);
    });
    // A link to `#fork=<id>` or `#joint=<id>` opens that case, so a chapter's
    // pointer lands on a record ready to run.
    #[cfg(feature = "desktop")]
    let session = use_context::<crate::Session>();
    use_effect(move || {
        #[cfg(feature = "desktop")]
        let fragment = (session.fragment)();
        #[cfg(not(feature = "desktop"))]
        let fragment = location_fragment();
        open_link(state, cancellation, &fragment);
    });
    use_effect(move || {
        let Some(job) = state.read().job() else {
            return;
        };
        let flag = Arc::new(AtomicBool::new(false));
        cancellation.set(flag.clone());
        let (request, selection) = {
            let mut s = state.write();
            s.active = true;
            s.background = job.background;
            s.pending = None;
            s.request += 1;
            if !job.background {
                s.phase = Phase::Running;
                s.message = "Executing this record in an isolated knowledge base…".into();
            }
            (s.request, s.selection_id)
        };
        spawn(async move {
            let result = reasoning::execute(job.id.clone(), flag.clone()).await;
            if flag.load(Ordering::Relaxed)
                || state.peek().request != request
                || state.peek().selection_id != selection
            {
                return;
            }
            {
                let mut s = state.write();
                s.active = false;
                match result {
                    Ok(response) if valid(&response.outcome, &job.id) => {
                        s.accept(&job, response.outcome);
                        if !job.background {
                            s.phase = Phase::Complete;
                            s.message = format!(
                                "Executed locally in {:.2} s. A derived conclusion is not an observed event.",
                                response.elapsed_ms / 1000.0
                            );
                        }
                    }
                    Ok(response) => {
                        if job.background {
                            s.restore_paused = true;
                        } else {
                            s.phase = Phase::Incomplete;
                        }
                        s.message = format!(
                            "Incomplete response for {} ({} returned queries). No completion or tally awarded. Retry to execute again.",
                            job.id,
                            response.outcome.verdicts.len()
                        );
                    }
                    Err(e) => {
                        if job.background {
                            s.restore_paused = true;
                        } else {
                            s.phase = Phase::Failed;
                        }
                        s.message = format!(
                            "Execution failed for {}: {e}. Retry to execute again.",
                            job.id
                        );
                    }
                }
            }
            save(state);
        });
    });
    use_drop(move || reasoning::cancel(&cancellation.peek()));
    let s = state.read().clone();
    let fork = game().scenarios.iter().find(|f| f.id == s.selection);
    let joint = game().joints.iter().find(|j| j.id == s.selection);
    let person = game().people.iter().find(|p| p.id == s.person).unwrap();
    let tally = s.tally();
    let unlocked = game().faults.iter().filter(|f| s.unlocked(&f.id)).count();
    let working = s.active && !s.background;
    let shown = if s.phase == Phase::Complete {
        s.displayed.last()
    } else {
        None
    };
    let record_id = s
        .pending
        .as_ref()
        .filter(|j| !j.background)
        .map(|j| j.id.clone())
        .or_else(|| s.displayed.last().map(|o| o.id.clone()))
        .or_else(|| ids(&s.selection).first().cloned());
    let record = record_id
        .as_ref()
        .and_then(|id| cases().cases.iter().find(|c| c.id == *id));
    let title = fork
        .map(|f| f.title.as_str())
        .or_else(|| joint.map(|j| j.title.as_str()))
        .unwrap_or("");
    let cost = fork
        .map(|f| &f.cost)
        .or_else(|| joint.map(|j| &j.cost))
        .unwrap();
    let cost = cost.clone();
    let source = fork
        .map(|f| f.source.as_str())
        .or_else(|| joint.map(|j| j.source.as_str()))
        .unwrap_or("");
    let objection = format!(
        "{REPOSITORY}/issues/new?title={}&body={}",
        encode_url(&format!("Companion: {title}")),
        encode_url(&format!(
            "Fork or joint: {}\nSource: {}\n\nMy objection or correction:\n\nEvidence or proposed change:\n",
            s.selection, source
        ))
    );
    rsx! {
        div { class:"game", "data-phase":s.phase.label(), "data-selection":"{s.selection}",
            div {class:"tally-bar",div{class:"container tally-inner",
                span{class:"eyebrow","// a life · book 1"}
                span{class:"save-person","{person.name}"}
                div{class:"tallies",aria_label:"Design tally from completed live records",
                    span{class:"held","held " b{"data-tally":"held","{tally[0]}"}}
                    span{class:"limited","limited " b{"data-tally":"limited","{tally[1]}"}}
                    span{class:"fault","fault " b{"data-tally":"fault","{tally[2]}"}}
                }
                NavLink{to:format!("{PREFIX}#dossier"),"dossier {unlocked}/{game().faults.len()} →"}
                button{class:"q-btn q-btn--ghost",onclick:move |_|{
                    let url=format!("{ORIGIN}{PREFIX}#game={}",encode_url(&serde_json::to_string(&state.peek().history).unwrap()));
                    share.set(url.clone());
                    spawn(async move {let mut eval=document::eval("const link=await dioxus.recv();try{await navigator.clipboard.writeText(link);dioxus.send(true)}catch(_){dioxus.send(false)}");let _=eval.send(url);let copied=eval.recv::<bool>().await.unwrap_or(false);copy_label.set(if copied{"Copied"}else{"Select link below"}.into());});
                },"{copy_label}"}
                button{class:"q-btn q-btn--ghost",onclick:move |_|{
                    reasoning::cancel(&cancellation.peek());let loaded=state.peek().loaded;let request=state.peek().request+1;
                    let mut reset=Play::default();reset.loaded=loaded;reset.request=request;reset.selection_id=state.peek().selection_id+1;
                    if loaded {reset.phase=Phase::Ready;reset.message="Game progress cleared. Choose a move.".into();}
                    state.set(reset);share.set(String::new());save(state);
                    #[cfg(feature="web")] {let _=document::eval("if(location.hash.startsWith('#game='))history.replaceState(null,'',location.pathname);");}
                    if !loaded {boot(state,cancellation);}
                },"Reset game ↻"}
            }}
            section{class:"hero game-hero",div{class:"hero__grid"}div{class:"container",
                div{class:"hero__eyebrow",span{class:"eyebrow","// one person, four lenses"}span{class:"q-badge engine-status","engine · {s.phase.label()}"}}
                h1{"Pick a person. Walk their " em{"forks"} "."}
                p{class:"hero__tagline","Each fork asks what this design does for a person — through their life, an adversary, an office or a changed rule. The tally scores the design, never the person."}
                div{class:"game-meta",NavLink{to:format!("{PREFIX}read/"),"Read the book →"}a{href:REPOSITORY,"Inspect the source ↗"}span{"Local execution · Nibli " {&cases().engine_revision[..12]}}}
                noscript{p{class:"note","Gameplay requires JavaScript and local engine execution. The complete book, chapter links, sources and Markdown remain available without JavaScript."}}
            }}
            div{class:"container game-body",
                if !share().is_empty(){div{class:"share-link",label{r#for:"share-game","Shareable history · recipients check it by running it locally"}input{id:"share-game",readonly:true,value:"{share}",onclick:move |_|{let _=document::eval("document.getElementById('share-game')?.select()");}}}}
                if !s.storage.is_empty(){p{class:"storage-notice",role:"status","{s.storage}"}}
                if !s.restore.is_empty(){div{class:"restore-message",role:"status","Checking saved progress · {s.restore.len()} records remain. Totals show only completed checks."
                    if s.restore_paused {button{class:"q-btn q-btn--secondary",onclick:move |_|state.write().restore_paused=false,"Retry saved progress"}}
                }}
                nav{class:"people",aria_label:"Choose a person",for p in &game().people {{let id=p.id.clone();let first=game().scenarios.iter().find(|f|f.person==p.id).unwrap().id.clone();let count=game().scenarios.iter().filter(|f|f.person==p.id).count();
                    rsx!{button{class:if p.id==s.person{"person selected"}else{"person"},aria_pressed:p.id==s.person,"data-person":"{p.id}",onclick:move |_|{select(state,cancellation,&first);state.write().person=id.clone();},
                        span{class:"avatar",{p.id.chars().next().unwrap().to_string()}}span{strong{"{p.name}"}small{"{p.role}"}small{class:"mono","{count} forks"}}
                    }}
                }}}
                div{class:"game-layout",div{class:"game-main",
                    section{class:"q-card track-card",h2{class:"eyebrow","// {person.name} · the track"}
                        div{class:"lens-legend",for lens in &game().lenses{span{"{lens.glyph} {lens.label}"}}}
                        nav{class:"fork-track",aria_label:"Choose a fork",for f in game().scenarios.iter().filter(|f|f.person==s.person){{let id=f.id.clone();let lens=game().lenses.iter().find(|l|l.id==f.lens).unwrap();
                            rsx!{button{class:if f.id==s.selection{"fork selected"}else{"fork"},aria_pressed:f.id==s.selection,"data-fork":"{f.id}",onclick:move |_|select(state,cancellation,&id),
                                span{class:"fork-glyph","{lens.glyph}"}small{"{lens.label} · ch. {f.chapter}"}strong{"{f.title}"}
                                span{class:"q-badge",if s.receipts.contains_key(&f.id){"completed live"}else if s.history.completed.contains(&f.id){"checking history"}else{"unplayed"}}
                            }}
                        }}}
                    }
                    section{class:"q-card play-card", "data-play":"{s.selection}",
                        header{class:"play-header",div{span{class:"eyebrow",if let Some(f)=fork {"// {game().lenses.iter().find(|l|l.id==f.lens).unwrap().label}"}else{"// rewrite the joints"}}
                            h2{"{title}"}if let Some(f)=fork{p{"{f.role}"}NavLink{to:chapter(f.chapter).path.clone(),"Read chapter {f.chapter} →"}}
                        }}
                        div{class:"play-body",
                            p{class:"run-message",role:"status",aria_live:"polite","{s.message}"}
                            if matches!(s.phase,Phase::Loading)||working {div{class:"pending",role:"status","◌ " if s.phase==Phase::Loading{"Loading engine resources…"}else{"Running the record…"}}button{class:"q-btn q-btn--secondary",onclick:move |_|{
                                reasoning::cancel(&cancellation.peek());let mut s=state.write();s.request+=1;s.active=false;s.pending=None;s.phase=Phase::Cancelled;s.restore_paused=true;s.message="Cancelled. This move has no result. Retry starts a fresh knowledge base.".into();
                            },"Cancel"}}
                            for (i,outcome) in s.displayed.iter().enumerate() {{
                                let queries=fork.map(|f|f.steps[i].queries.clone()).or_else(||joint.map(|j|j.queries.clone())).unwrap_or_default();
                                let label=fork.map(|f|f.steps[i].label.clone()).unwrap_or_else(||if i==0{"Canonical constitution".into()}else{"Declared counterfactual".into()});
                                rsx!{div{class:"played-step","data-case":"{outcome.id}",h3{span{class:"step-number","{i+1:02}"}" {label}"}
                                    div{class:"verdict-grid",for query in queries {{let v=outcome.verdicts.iter().find(|v|v.query==query.text).unwrap();let t=tag(&query,&v.status);
                                        rsx!{div{class:"verdict","data-result-kind":"live",div{strong{"{query.label}"}p{class:"result-explanation",match v.status.as_str(){"TRUE"=>"Derivable from this record and these rules.","FALSE"=>"Not derivable from this record; this does not establish real-world absence.","REFUSED"=>"The engine refused this query. Refusal is distinct from a negative answer.",_=>"The engine did not complete this query."}}
                                            details{summary{"Query and engine detail"}code{"? {v.query}"}if let Some(detail)=&v.detail{p{"{detail}"}}}
                                            if !matches!(query.kind.as_str(),"context")&&t.is_none(){p{class:"neutral-result","This result contributes no editorial tally tag. The returned answer is shown unchanged."}}
                                        }div{class:"verdict-status",output{class:v.status.to_lowercase(),"{v.status}"}if let Some(t)=t{span{class:"q-badge","{t}"}}}}}
                                    }}}
                                }}
                            }}
                            if !s.loaded&&!matches!(s.phase,Phase::Loading){button{class:"q-btn q-btn--primary",onclick:move |_|boot(state,cancellation),"Retry engine startup"}}
                            if let Some(f)=fork {
                                if let Some(next)=f.steps.get(s.displayed.len()) {{let id=next.id.clone();rsx!{button{class:"q-btn q-btn--primary move-button",disabled:!s.loaded||working,onclick:move |_|queue(state,cancellation,id.clone()),
                                    if matches!(s.phase,Phase::Failed|Phase::Incomplete|Phase::Cancelled){"Retry · "}"{next.label} →"
                                }}}}
                                if !s.displayed.is_empty(){button{class:"q-btn q-btn--secondary",disabled:working,onclick:move |_|{
                                    let id=state.peek().selection.clone();select(state,cancellation,&id);queue(state,cancellation,ids(&id)[0].clone());
                                },"Run again · replay from start"}}
                                if s.displayed.len()==f.steps.len(){p{class:"completion",role:"status","✓ Fork completed by live execution. Replaying cannot add a duplicate tally."}}
                            }
                            if let Some(j)=joint {
                                p{class:"authored-label","Authored design choice"}
                                p{if *s.history.joints.get(&j.id).unwrap_or(&j.initial){"{j.title}"}else{"{j.off_label}"}}
                                {let id=j.id.clone();rsx!{button{class:"q-btn q-btn--secondary",aria_pressed:*s.history.joints.get(&j.id).unwrap_or(&j.initial),disabled:working,onclick:move |_|{
                                    let value=state.peek().history.joints.get(&id).copied().unwrap_or_else(||game().joints.iter().find(|j|j.id==id).unwrap().initial);
                                    state.write().history.joints.insert(id.clone(),!value);save(state);
                                    if game().joints.iter().find(|j|j.id==id).unwrap().measured {select(state,cancellation,&id);queue(state,cancellation,format!("{id}:canonical"));}
                                },if *s.history.joints.get(&j.id).unwrap_or(&j.initial){"Change this choice"}else{"Restore this choice"}}}}
                                if j.measured{{let id=j.id.clone();rsx!{button{class:"q-btn q-btn--primary",disabled:!s.loaded||working,onclick:move |_|{select(state,cancellation,&id);queue(state,cancellation,format!("{id}:canonical"));},if s.displayed.is_empty(){"Compare both records live"}else{"Run again · compare both"}}}}}
                                else{p{"This switch opens an authored cost. No executable alternative or verdict change is claimed."}}
                                if s.displayed.len()==2{p{class:"completion",if changed(&s.displayed){"Live comparison found a changed conclusion. One counterfactual fault is recorded."}else{"Live comparison completed without a changed definitive conclusion. No counterfactual fault is awarded."}}}
                            }
                            details{class:"record",summary{"Inspect the complete executable record"}
                                if let Some(record)=record{p{"A fresh knowledge base loads the full constitution, then these entries. The move’s short description is authored."}
                                    pre{if record.record.is_empty(){"// No additional premises"}else{{record.record.join("\n")}}}
                                    h3{"Source mappings"}ul{for source in &record.sources{li{a{href:format!("{REPOSITORY}/blob/main/{}{}",source.path,source.through_line.map(|n|format!("#L{n}")).unwrap_or_default()),"{source.path}"}}}}
                                }else{p{"This authored alternative has no executable record."}}
                            }
                            aside{class:"authored-cost",strong{"Authored cost · {cost.title}"}p{"{cost.text}"}}
                            a{href:objection,target:"_blank",rel:"noopener","Object to this fork ↗"}
                        }
                    }
                }
                aside{class:"game-sidebar",
                    section{class:"q-card floor-panel",h2{"The floor · {person.name}"}p{"Duties and provision in the current record."}
                        div{class:"floor-indicators",for (name,item,pred) in [("Food","Eats","eats"),("Shelter","Dwell","dwell"),("Care","Healthy","healthy"),("Learning","Learn","learn"),("Safety","Secure","secure"),("Material security","Suffice","suffice"),("Expression","Expresses","expresses"),("Belief","Believe","believe"),("Company","Meets","meets")]{{
                            let who=if s.person=="Newcomer"{"MPNewcomer"}else{&s.person};
                            let status=|q:String|if fork.is_some(){shown.and_then(|o|o.verdicts.iter().find(|v|v.query==q)).map(|v|v.status.as_str()).unwrap_or("pending")}else{"unqueried"};
                            let owed=status(format!("owe(State, {item}, {who})."));let delivered=status(format!("{pred}({who})."));
                            rsx!{div{class:"floor-indicator","data-floor":pred,strong{"{name}"}span{"owed · {owed}"}span{"provided · {delivered}"}}}
                        }}}
                        p{class:"term-note","Provision is a formal conclusion. Bodily safety, expression and belief have no delivery route by design."}
                    }
                    section{class:"q-card joint-panel",h2{"Rewrite the joints"}p{"Change a choice, examine its cost."}
                        for j in &game().joints{{let id=j.id.clone();rsx!{button{class:if s.selection==j.id{"joint selected"}else{"joint"},"data-joint":"{j.id}",aria_pressed:s.selection==j.id,onclick:move |_|select(state,cancellation,&id),span{"⇌"}span{strong{"{j.title}"}small{if j.measured{"live comparison"}else{"authored cost"}}}}}}}
                    }
                }}
                section{id:"dossier",class:"dossier",h2{"The dossier"}p{"Authored questions, admitted costs and objections. “Examined live” marks a completed record; it is not proof that the accompanying argument is correct. Measured entries require both executions and a changed conclusion."}
                    div{class:"dossier-grid",for fault in &game().faults{article{class:"q-card dossier-entry","data-dossier":"{fault.id}",header{span{class:"q-badge",if s.unlocked(&fault.id){"examined live"}else{"awaiting execution"}}small{"{fault.kind}"}}h3{"{fault.title}"}p{"{fault.text}"}p{class:"term-note","Authored source · {fault.book}"}
                        a{href:format!("{REPOSITORY}/issues/new?title={}&body={}",encode_url(&format!("Companion dossier: {}",fault.title)),encode_url(&format!("Entry: {}\n\nMy objection or evidence:\n",fault.id))),target:"_blank",rel:"noopener","Raise an objection ↗"}
                    }}}
                }
                section{id:"sources",class:"q-card game-sources",h2{"Sources, limits and reading"}
                    p{"This is a constitutional design offered for criticism. Test names are formal records, not biographies. TRUE means derivable; FALSE means not derivable; REFUSED means the query was not admitted. Failure, cancellation and incomplete execution award no result. Tally tags are authored interpretations of returned answers."}
                    details{summary{"How the tally works"}p{"Each completed fork contributes once. A protection that derives, a challenged consequence that does not derive, or refused status vocabulary can count as held. Unestablished provision or an unfulfilled-duty discussion can count as limited. An unestablished standing root can count as a record fault. A measured joint contributes a fault only when the two live executions differ. These tags assess selected records, not the person or an operating society."}}
                    p{"Opening the game downloads the pinned engine and full compiled constitution. Execution and history stay on this device. Saved or shared history must be replayed before it contributes. Reader pages start no engine. Book 2 concerns operation and transition and remains inactive until Book 1’s release decision."}
                    div{class:"actions",NavLink{to:format!("{PREFIX}read/"),"Complete reader →"}a{href:public_url(format!("{PREFIX}game.json")),"Game data"}a{href:public_url(format!("{PREFIX}cases.json")),"Executable inputs"}a{href:format!("{REPOSITORY}/blob/main/book-1/source/constitution.nibli"),"Constitution"}a{href:format!("{REPOSITORY}/blob/main/LICENSING.md"),"Licences"}a{href:public_url(format!("{PREFIX}assets/fonts/README.md")),"Bundled fonts"}}
                    p{"Prose: CC BY 4.0 · code: MIT or Apache-2.0 · constitutional inputs: CC0. QUINE typography uses IBM Plex and Space Grotesk, with Noto Serif Tamil for the epigraph."}
                }
            }
        }
    }
}
