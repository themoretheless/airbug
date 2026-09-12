import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const template=fs.readFileSync('crates/rbench/src/memory-template.html','utf8');
class Element {
  constructor(){this.children=[];this.style={};this.value='';this.textContent='';this.attrs={};}
  append(...items){this.children.push(...items)}
  replaceChildren(...items){this.children=items}
  setAttribute(k,v){this.attrs[k]=v}
}
const els=new Map();const get=id=>{if(!els.has(id))els.set(id,new Element());return els.get(id)};
get('data').textContent=JSON.stringify({events:3,dropped_events:2,peak_tracked_bytes:20,scope:'test',sites:[{frames:['root','a'],allocated_bytes:20,allocations:2,live_bytes:0,live_blocks:0},{frames:['root','b'],allocated_bytes:10,allocations:1,live_bytes:10,live_blocks:1}]});
get('metric').value='allocated_bytes';
vm.runInNewContext(template.split('</script><script>')[1].split('</script>')[0],{document:{getElementById:get,createElement:()=>new Element(),createElementNS:()=>new Element()}});
assert.equal(get('rows').children.length,2);
assert.ok(get('warning').textContent.includes('2'));
let svg=get('graph').children[0];assert.ok(svg.children[0].children[1].textContent.includes('30'));
get('metric').value='live_bytes';get('metric').onchange();
svg=get('graph').children[0];assert.ok(svg.children[0].children[1].textContent.includes('10'));
get('search').value='missing';get('search').oninput();assert.equal(get('rows').children.length,0);
assert.equal(get('graph').children.length,0);
get('search').value='b';get('search').oninput();assert.equal(get('rows').children.length,1);
get('graph').children[0].children.at(-1).onclick();assert.equal(get('rows').children.length,1);
get('reset').onclick();assert.equal(get('rows').children.length,1);
console.log('Memory DOM fixture: sums, live metric, missing events, search, zoom/reset passed');
