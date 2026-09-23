import json,sys,time
from host import call,out

def shader(language,ty,annotation,body):
    if language==1:
        return '#version 450\n// @param value '+annotation+'\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;'+ty+' value;};\nvoid main(){'+body+'}'
    return '// @param value '+annotation+'\nstruct FxUniforms {u_resolution:vec2f,u_time:f32,u_frame:f32,value:'+ty+',};\n@group(0) @binding(2) var<uniform> fx:FxUniforms;\n@fragment fn main()->@location(0) vec4f {'+body+'}'

cases=[]
for lang in [1,2]:
    for kind,annotation in [('rgb','hint:color default:#4080FF'),('rgba','hint:color default:#4080FF'),('alpha','hint:color default:#4080FF80'),('percent','hint:percent min:0 max:100 default:37.125')]:
        if lang==1:
            ty='float' if kind=='percent' else 'vec3' if kind=='rgb' else 'vec4'
            body='outColor=vec4(value/100.0,0,0,1);' if kind=='percent' else 'outColor=vec4(value,1);' if kind=='rgb' else 'outColor=vec4(value.rgb*value.a,value.a);'
        else:
            ty='f32' if kind=='percent' else 'vec3f' if kind=='rgb' else 'vec4f'
            body='return vec4f(fx.value/100.0,0,0,1);' if kind=='percent' else 'return vec4f(fx.value,1);' if kind=='rgb' else 'return vec4f(fx.value.rgb*fx.value.a,fx.value.a);'
        cases.append({'name':f'{lang}-{kind}','language':lang,'source':shader(lang,ty,annotation,body)})

if sys.argv[1]=='create':
    path=(out/'Parameter acceptance.aep').as_posix()
    body='if(!app.project.file||decodeURI(app.project.file.name)!=="reported-project-copy.aep")throw Error("Unexpected project");app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);app.newProject();app.project.bitsPerChannel=32;var cases='+json.dumps(cases)+';var rows=[];'
    body+='for(var k=0;k<cases.length;k++){var t=cases[k],c=app.project.items.addComp(t.name,64,64,1,2,25),l=c.layers.addSolid([1,1,1],"Material",64,64,1,2),e=l.property("ADBE Effect Parade").addProperty("DynamicFx");e.property("Language").setValue(t.language);e.property(3).expression="`"+t.source+"`;0";var n=c.layers.addNull();n.name="Sampler";n.enabled=false;n.property("ADBE Effect Parade").addProperty("ADBE Slider Control");rows.push({name:t.name,id:c.id});}app.project.save(new File('+json.dumps(path)+'));return JSON.stringify(rows);'
    call('parameter-create','(function(){'+body+'})()');time.sleep(3)
elif sys.argv[1]=='read':
    body='if(!app.project.file||decodeURI(app.project.file.name)!=="Parameter acceptance.aep")throw Error("Unexpected project");var rows=[];app.purge(PurgeTarget.ALL_CACHES);for(var k=1;k<=app.project.numItems;k++){var c=app.project.item(k);if(!(c instanceof CompItem))continue;var l=c.layer("Material"),e=l.property("ADBE Effect Parade").property(1),p=c.layer("Sampler").property("ADBE Effect Parade").property(1).property(1),rgba=[];for(var j=0;j<4;j++){p.expression="thisComp.layer(\\\"Material\\\").sampleImage([32,32],[0.49,0.49],true,0)["+j+"]";rgba.push(p.value);if(p.expressionError)throw Error(p.expressionError);}var values=[];for(var j=1;j<=e.numProperties;j++)if(e.property(j).name==="value"||e.property(j).name==="value A")values.push({name:e.property(j).name,index:j,value:e.property(j).value});rows.push({name:c.name,token:e.property(6).value,status:e.property(5).name,rgba:rgba,values:values});}app.project.save();return JSON.stringify(rows);'
    rows=call('parameter-read-'+sys.argv[2],'(function(){'+body+'})()')
    (out/('parameter-summary-'+sys.argv[2]+'.json')).write_text(json.dumps(rows,indent=2))
elif sys.argv[1]=='quit':
    call('candidate-quit','(function(){if(!app.project.file||decodeURI(app.project.file.name)!=="Parameter acceptance.aep")throw Error("Unexpected project");app.project.save();app.scheduleTask("app.quit()",500,false);return JSON.stringify("scheduled");})()')
