import ctypes,json,os,subprocess,sys,time
from pathlib import Path
out=Path(__file__).parent;dest=out/'cancel-close-frames';dest.mkdir(exist_ok=True)
main=Path(os.environ['TEMP'])/'dynamicfx.log';offset=main.stat().st_size
env=os.environ.copy();env['DYNAMICFX_RENDER_TRACE']='1';env['DYNAMICFX_VERBOSE_LOG']='1'
args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',str((out/'cancel.aep').resolve()),'-rqindex','1','-output',str((dest/'frame_[#####].psd').resolve()),'-mfr','ON','50','-v','ERRORS_AND_PROGRESS']
startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
with (out/'cancel-close.stdout.log').open('wb') as stdout,(out/'cancel-close.stderr.log').open('wb') as stderr:
    process=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=startup,env=env)
    print(json.dumps({'launcher_pid':process.pid,'started':True}),flush=True)
    deadline=time.monotonic()+90
    while time.monotonic()<deadline and process.poll() is None:
        with main.open('rb') as f:f.seek(offset);raw=f.read()
        if raw.count(b'native frame digest:')>=4:break
        time.sleep(.05)
    if process.poll() is not None:raise RuntimeError('Render ended before interruption')
    import re
    engine=int(re.findall(rb'native frame digest: pid=(\d+)',raw)[-1])
    user=ctypes.WinDLL('user32',use_last_error=True)
    user.GetWindowThreadProcessId.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ulong)]
    user.GetClassNameW.argtypes=[ctypes.c_void_p,ctypes.c_wchar_p,ctypes.c_int]
    user.PostMessageW.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_size_t,ctypes.c_ssize_t]
    handles=[]
    callback_type=ctypes.WINFUNCTYPE(ctypes.c_bool,ctypes.c_void_p,ctypes.c_ssize_t)
    @callback_type
    def visit(hwnd,param):
        pid=ctypes.c_ulong();user.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
        if pid.value==engine:
            name=ctypes.create_unicode_buffer(256);user.GetClassNameW(hwnd,name,256);handles.append((int(hwnd),name.value))
        return True
    user.EnumWindows(visit,0)
    (out/'cancel-close-windows.json').write_text(json.dumps({'engine':engine,'windows':handles}),encoding='utf-8')
    targets=[h for h,name in handles if name=='AE_CApplication_26.5' or name.startswith('AE_CApplication')]
    if len(targets)!=1:raise RuntimeError('Expected one owned engine application window: '+repr(handles))
    sent=user.PostMessageW(targets[0],0x10,0,0)
    if not sent:raise ctypes.WinError(ctypes.get_last_error())
    try:code=process.wait(timeout=45)
    except subprocess.TimeoutExpired:
        (out/'cancel-close-pending.json').write_text(json.dumps({'launcher_pid':process.pid,'signal_sent':True}),encoding='utf-8')
        raise
with main.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/'cancel-close.main.log').write_text(raw,encoding='utf-8')
report={'returncode':code,'close_sent':bool(sent),'interrupt_cancel_entries':raw.count('InterruptCancel'),'external_cancel_entries':raw.count('cancelled; discarding frame'),'frames_written':len(list(dest.glob('*.psd')))}
report['status']='PASS' if report['interrupt_cancel_entries']+report['external_cancel_entries']>0 and report['frames_written']<750 else 'UNVERIFIED_CANCEL'
(out/'cancel-close-summary.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report),flush=True)
