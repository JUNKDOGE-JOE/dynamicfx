import ctypes,json,os,subprocess,sys,time
from pathlib import Path
out=Path(__file__).parent;dest=out/'cancel-frames';dest.mkdir(exist_ok=True)
main=Path(os.environ['TEMP'])/'dynamicfx.log';offset=main.stat().st_size
env=os.environ.copy();env['DYNAMICFX_RENDER_TRACE']='1';env['DYNAMICFX_VERBOSE_LOG']='1'
args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',str((out/'cancel.aep').resolve()),'-rqindex','1','-output',str((dest/'frame_[#####].psd').resolve()),'-mfr','ON','50','-v','ERRORS_AND_PROGRESS']
startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
with (out/'cancel.stdout.log').open('wb') as stdout,(out/'cancel.stderr.log').open('wb') as stderr:
    process=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=startup,env=env)
    print(json.dumps({'launcher_pid':process.pid,'started':True}),flush=True)
    deadline=time.monotonic()+90
    while time.monotonic()<deadline and process.poll() is None:
        with main.open('rb') as f:f.seek(offset);raw=f.read()
        if raw.count(b'native frame digest:')>=4:break
        time.sleep(.05)
    if process.poll() is not None:raise RuntimeError('Render ended before interruption')
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.FreeConsole()
    attached=kernel.AttachConsole(process.pid)
    if not attached:raise ctypes.WinError(ctypes.get_last_error())
    kernel.SetConsoleCtrlHandler(None,True)
    sent=kernel.GenerateConsoleCtrlEvent(0,0)
    error=ctypes.get_last_error()
    time.sleep(.2)
    kernel.FreeConsole()
    if not sent:raise ctypes.WinError(error)
    try:code=process.wait(timeout=45)
    except subprocess.TimeoutExpired:
        (out/'cancel-pending.json').write_text(json.dumps({'launcher_pid':process.pid,'signal_sent':True}),encoding='utf-8')
        raise
with main.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/'cancel.main.log').write_text(raw,encoding='utf-8')
report={'returncode':code,'ctrl_c_sent':bool(sent),'interrupt_cancel_entries':raw.count('InterruptCancel'),'external_cancel_entries':raw.count('cancelled; discarding frame'),'frames_written':len(list(dest.glob('*.psd')))}
report['status']='PASS' if report['interrupt_cancel_entries']+report['external_cancel_entries']>0 and report['frames_written']<750 else 'UNVERIFIED_CANCEL'
(out/'cancel-summary.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report),flush=True)
