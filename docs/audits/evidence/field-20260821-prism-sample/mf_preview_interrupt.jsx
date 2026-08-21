// Reproduce the user's interactive workflow: play the comp preview, interrupt it by
// moving the CTI, play again -- N cycles -- while counting the plug-in's
// "input checkout failed" lines. State lives on $.global so scheduleTask can drive it.
var OUT = "C:/Users/A/AppData/Local/Temp/claude/E--Code-AePlugin-Dynamicfx/714697b6-beff-43bc-80eb-f198d43397c3/scratchpad/mf_preview";
var outFolder = new Folder(OUT);
if (!outFolder.exists) outFolder.create();
$.global.__pi = {
  comp: app.project.itemByID(1),
  start: 3.0,           // seconds; work area [3.0, 7.0) = frames 90..209
  span: 4.0,
  cycle: 0,
  maxCycles: 8,
  phase: "play",
  playMs: 1400,
  log: [],
  cmdPlay: 10314       // '播放当前预览' (Play Current Preview) in this UI language
};
$.global.__piCountLog = function () {
  var f = new File(Folder.temp.fsName + "/dynamicfx.log");
  if (!f.exists) return -1;
  f.encoding = "UTF-8";
  f.open("r");
  var n = 0, m = 0;
  while (!f.eof) { var l = f.readln(); if (l.indexOf("checkout failed") >= 0) n++; m++; }
  f.close();
  return n + "/" + m;
};
$.global.__piStep = function () {
  var S = $.global.__pi;
  try {
    if (S.phase == "play") {
      // Interrupt point for the previous cycle: moving the CTI stops playback and cancels in-flight renders.
      var jitter = (S.cycle % 4) * 0.37;
      S.comp.time = S.start + 0.5 + jitter;
      S.log.push("cycle " + S.cycle + " t=" + new Date().getTime() + " stop->time " + (S.start + 0.5 + jitter).toFixed(2) + " log=" + $.global.__piCountLog());
      if (S.cycle >= S.maxCycles) {
        S.phase = "done";
        var done = new File(OUT + "/done.txt");
        done.encoding = "UTF-8";
        done.open("w");
        done.write(S.log.join("\n"));
        done.close();
        return;
      }
      S.comp.time = S.start + jitter * 0.5;
      app.executeCommand(S.cmdPlay);
      S.log.push("cycle " + S.cycle + " t=" + new Date().getTime() + " play from " + S.comp.time.toFixed(2));
      S.cycle++;
      app.scheduleTask("$.global.__piStep()", S.playMs, false);
    }
  } catch (e) {
    S.log.push("error: " + e);
    var done2 = new File(OUT + "/done.txt");
    done2.open("w"); done2.write(S.log.join("\n")); done2.close();
  }
};
var S = $.global.__pi;
S.comp.openInViewer();
S.comp.workAreaStart = S.start;
S.comp.workAreaDuration = S.span;
app.purge(PurgeTarget.ALL_CACHES);
S.log.push("begin log=" + $.global.__piCountLog());
app.scheduleTask("$.global.__piStep()", 300, false);
"scheduled; work area " + S.start + "+" + S.span + "; initial log count " + $.global.__piCountLog();
