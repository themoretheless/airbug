"""Terminal acceptance: run a direct harness in a PTY at 80 and 120 columns in every console
mode, report lines wider than the terminal, and render the captured output (colours included)
as an HTML page for a look.

    python3 bench/scripts/terminal-acceptance.py out.html target/release/examples/sort
"""
import os, pty, fcntl, termios, struct, subprocess, select, html, re, sys, json
exe=os.path.abspath(sys.argv[2])
base=[exe,'--no-history','--samples','3','--warmup-ms','0','--resamples','32','--color','always']
modes={'default':[], 'quiet':['--quiet'], 'verbose':['--verbose'], 'tree':['--output-format','tree'], 'list-tree':['--output-format','tree','--list']}
def run(args,width):
    m,s=pty.openpty(); fcntl.ioctl(s,termios.TIOCSWINSZ,struct.pack('HHHH',40,width,0,0))
    p=subprocess.Popen(args,stdin=s,stdout=s,stderr=s,env=dict(os.environ,AIRBUG_DASHBOARD='0',TERM='xterm-256color'),cwd=os.path.dirname(os.path.abspath(sys.argv[1])) or '.'); os.close(s); out=b''
    while True:
        r,_,_=select.select([m],[],[],60)
        if not r: p.kill(); break
        try: d=os.read(m,65536)
        except OSError: break
        if not d: break
        out+=d
    p.wait(); return p.returncode,out.decode('utf-8','replace')
COL={30:'#000',31:'#c33',32:'#2a8',33:'#b80',34:'#36c',35:'#a4a',36:'#2aa',37:'#ccc',90:'#777',91:'#f66',92:'#5d5',93:'#fc5',94:'#69f',95:'#f7f',96:'#6ee',97:'#fff'}
def to_html(text):
    text=text.replace('\r\n','\n'); out=[]; style={}; i=0; opened=False
    for part in re.split(r'(\x1b\[[0-9;]*m)',text):
        m=re.fullmatch(r'\x1b\[([0-9;]*)m',part)
        if m:
            for code in [int(c or 0) for c in m.group(1).split(';')]:
                if code==0: style={}
                elif code==1: style['font-weight']='bold'
                elif code==2: style['opacity']='.6'
                elif code in COL: style['color']=COL[code]
            continue
        part=re.sub(r'\x1b\[[0-9;?]*[A-Za-z]','',part)
        # carriage returns redraw a line: keep the last version
        part='\n'.join(l.split('\r')[-1] for l in part.split('\n'))
        css=';'.join(f'{k}:{v}' for k,v in style.items())
        out.append(f'<span style="{css}">{html.escape(part)}</span>' if css else html.escape(part))
    return ''.join(out)
rows=[]; pages=[]
for name,extra in modes.items():
    for width in (80,120):
        rc,text=run(base+extra,width)
        plain=re.sub(r'\x1b\[[0-9;?]*[A-Za-z]','',text).replace('\r\n','\n')
        lines=[l.split('\r')[-1] for l in plain.split('\n')]
        longest=max((len(l) for l in lines),default=0)
        rows.append(dict(mode=name,width=width,rc=rc,lines=len(lines),longest=longest,over=sum(len(l)>width for l in lines)))
        pages.append(f'<h3>{name} · {width} columns · exit {rc}</h3><pre style="width:{width}ch">{to_html(text)}</pre>')
open(sys.argv[1],'w').write('<!doctype html><meta charset=utf-8><style>body{background:#111;color:#ddd;font:13px/1.35 monospace;padding:16px}pre{border:1px solid #444;padding:0;margin:0 0 24px;white-space:pre-wrap;word-break:break-all;font:inherit}h3{color:#8cf;font:13px monospace}</style>'+''.join(pages))
print(json.dumps(rows,indent=1))
