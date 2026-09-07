var c=comp("DFX_aerender_fresh");var s=state(c);if(s.token%4!==1)throw Error("Fresh not ready");
app.project.bitsPerChannel=8;app.purge(PurgeTarget.ALL_CACHES);
var rq=app.project.renderQueue;while(rq.numItems>0)rq.item(1).remove();
var i=rq.items.add(c);i.timeSpanStart=0;i.timeSpanDuration=1/25;
var om=i.outputModule(1);om.applyTemplate("Photoshop");om.file=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/fresh_[#####].psd");
app.project.save(new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/native-aerender-fresh.aep"));return s;