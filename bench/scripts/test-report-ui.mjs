// Unit-test embedded report controls using a tiny DOM fixture; no browser/visual claim.
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const html=fs.readFileSync(new URL('../crates/rbench/src/report-template.html',import.meta.url),'utf8');
const script=html.match(/<script type="text\/javascript">([\s\S]*?)<\/script>/)[1];
const element=(text='')=>({textContent:text,hidden:false,value:'',dataset:{},events:{},addEventListener(k,f){this.events[k]=f;},closest(){return null;}});
const ids=Object.fromEntries(['search','outcome','count','collection-tools','expand','collapse','print'].map(k=>[k,element()]));
const articles=[element('slow latency scope'),element('stable parsing')];articles[0].dataset.outcome='regression';articles[1].dataset.outcome='uncompared';
const rows=[element('slow 20'),element('stable 10'),element('slow latency scope'),element('stable parsing')];rows[0].dataset.outcome='regression';rows[1].dataset.outcome='uncompared';rows[2].closest=()=>articles[0].hidden?articles[0]:null;rows[3].closest=()=>articles[1].hidden?articles[1]:null;
rows[0].cells=[element('slow'),element('20')];rows[1].cells=[element('stable'),element('10')];
const details=[element(),element()];details[0].open=false;details[1].open=false;details[0].closest=()=>articles[0].hidden?articles[0]:null;details[1].closest=()=>articles[1].hidden?articles[1]:null;
const anchor=element();let printed=0;
const th=[element(),element()];const button=element();button.parentElement=th[1];
const tbody={rows:[rows[0],rows[1]],appendChild(row){this.rows=this.rows.filter(r=>r!==row);this.rows.push(row);}};
const table={tBodies:[tbody],querySelectorAll(){return th;}};
for(const h of th){h.parentElement={children:th};h.closest=()=>table;h.attrs={};h.getAttribute=k=>h.attrs[k];h.setAttribute=(k,v)=>h.attrs[k]=v;h.removeAttribute=k=>delete h.attrs[k];}
const document={getElementById:id=>ids[id],querySelector:()=>articles[0],querySelectorAll(selector){switch(selector){case 'tbody tr':return rows;case 'article[data-outcome]':return articles;case 'a[href^="#run-"]':return [anchor];case 'th button':return [button];case 'details':return details;case 'details:not([open])':return details.filter(d=>!d.open);case 'article:not([hidden]) details':return details.filter(d=>!d.closest());default:throw Error(selector);}}};
vm.runInNewContext(script,{document,window:{print(){printed++;assert(details.every(d=>d.open||d.closest()));}}});
assert.equal(ids.count.textContent,'4 / 4 rows');assert.equal(ids['collection-tools'].hidden,false);
ids.outcome.value='regression';ids.outcome.events.change();assert.equal(articles[1].hidden,true);assert.equal(rows[3].hidden,true);assert.equal(ids.count.textContent,'2 / 4 rows');
ids.expand.events.click();assert.equal(details[0].open,true);assert.equal(details[1].open,false);ids.collapse.events.click();assert(details.every(d=>!d.open));
ids.search.value='not present';ids.search.events.input();assert(articles.every(a=>a.hidden));assert.equal(ids.count.textContent,'0 / 4 rows');
anchor.events.click();assert(articles.every(a=>!a.hidden));assert.equal(ids.search.value,'');assert.equal(ids.outcome.value,'');
button.events.click();assert.equal(tbody.rows[0].cells[1].textContent,'10');button.events.click();assert.equal(tbody.rows[0].cells[1].textContent,'20');
ids.print.events.click();assert.equal(printed,1);assert(details.every(d=>!d.open));
console.log('Report controls: search, outcomes, visible counts, navigation, numeric sort, expand/collapse and print restoration passed. DOM fixture only.');
