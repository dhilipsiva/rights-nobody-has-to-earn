// SPDX-License-Identifier: MIT OR Apache-2.0
// Contrast of rendered text against its composited solid backgrounds. Decorative
// gradients and disabled controls are excluded; this is not a WCAG certification.
() => {
  const ctx=document.createElement('canvas').getContext('2d',{willReadFrequently:true});
  const rgba=color=>{ctx.clearRect(0,0,1,1);ctx.fillStyle=color;ctx.fillRect(0,0,1,1);return [...ctx.getImageData(0,0,1,1).data].map((x,i)=>i===3?x/255:x);};
  const over=(fg,bg)=>fg.slice(0,3).map((x,i)=>x*fg[3]+bg[i]*(1-fg[3])).concat(1);
  const background=el=>{
    const parents=[];for(let node=el;node;node=node.parentElement)parents.push(node);
    return parents.reverse().reduce((bg,node)=>over(rgba(getComputedStyle(node).backgroundColor),bg),[255,255,255,1]);
  };
  const luminance=rgb=>rgb.slice(0,3).map(x=>x/255).map(x=>x<=.04045?x/12.92:((x+.055)/1.055)**2.4).reduce((n,x,i)=>n+x*[.2126,.7152,.0722][i],0);
  let checked=0,min=100;const failures=[];
  for(const el of document.querySelectorAll('body *')){
    if(![...el.childNodes].some(n=>n.nodeType===3&&n.textContent.trim())||!el.getClientRects().length||el.closest('[disabled],.sr-only,.skip-link'))continue;
    const css=getComputedStyle(el);
    if(css.visibility!=='visible'||css.opacity==='0')continue;
    const bg=background(el),fg=over(rgba(css.color),bg),a=luminance(fg),b=luminance(bg);
    const ratio=(Math.max(a,b)+.05)/(Math.min(a,b)+.05);
    const large=parseFloat(css.fontSize)>=24 || (parseFloat(css.fontSize)>=18.66 && parseInt(css.fontWeight)>=700);
    if(ratio<(large?3:4.5))failures.push({text:el.textContent.trim().slice(0,70),selector:el.className||el.tagName,ratio:Math.round(ratio*100)/100});
    checked++;min=Math.min(min,ratio);
  }
  return {checked,minimum:Math.round(min*100)/100,failures};
}
