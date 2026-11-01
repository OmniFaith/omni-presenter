//! JavaScript render/event modules share persistent JSON state across native and web outputs.
use super::*;
use rquickjs::{CatchResultExt, Context, Runtime};
use std::{cell::{Cell, RefCell},collections::HashMap,rc::Rc,time::{Duration,Instant}};

struct Engine { context:Context, _runtime:Runtime, deadline:Rc<Cell<Instant>> }
thread_local! {static ENGINES:RefCell<HashMap<String,Engine>>=RefCell::new(HashMap::new());}

fn evaluate(source:&str,input:&Value,operation:&str)->Result<Value,String>{
    if source.len()>262_144{return Err("plugins-too-large".into());}
    ENGINES.with(|cache|{
        let mut cache=cache.borrow_mut();
        if !cache.contains_key(source){
            if cache.len()>=24{cache.clear();}
            let runtime=Runtime::new().map_err(|e|e.to_string())?;
            runtime.set_memory_limit(32*1024*1024);runtime.set_max_stack_size(512*1024);
            let deadline=Rc::new(Cell::new(Instant::now()+Duration::from_millis(100)));
            let limit=deadline.clone();runtime.set_interrupt_handler(Some(Box::new(move||Instant::now()>limit.get())));
            let context=Context::full(&runtime).map_err(|e|e.to_string())?;
            context.with(|ctx|->Result<(),String>{
                ctx.eval::<(),_>(include_str!("../../../assets/plugins/canvas-runtime.js")).catch(&ctx).map_err(|e|e.to_string())?;
                ctx.eval::<(),_>(format!("globalThis.__moduleFactory = () => ({source});")).catch(&ctx).map_err(|e|e.to_string())
            })?;
            cache.insert(source.into(),Engine{context,_runtime:runtime,deadline});
        }
        let engine=cache.get(source).unwrap();engine.deadline.set(Instant::now()+Duration::from_millis(35));
        let result=engine.context.with(|ctx|->Result<String,String>{
            ctx.globals().set("__input",serde_json::to_string(input).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            ctx.eval::<String,_>(format!("JSON.stringify((()=>{{const api=JSON.parse(__input);const plugin=__moduleFactory();{operation}}})())")).catch(&ctx).map_err(|e|e.to_string())
        })?;
        if result.len()>1_048_576{return Err("plugins-too-large".into());}
        serde_json::from_str(&result).map_err(|e|e.to_string())
    })
}

pub fn validate_expression(source:&str)->Result<(),String>{evaluate(source,&serde_json::json!({}),"return true;").map(|_|())}
pub fn validate(component:&JavaScriptComponent)->Result<(),String>{
    evaluate(&component.source,&serde_json::json!({"props":{},"state":component.initial_state,"now":0,"width":640,"height":360}),
        "if(!plugin || typeof plugin.render!=='function')throw Error('render(api, ctx) is required');return true;").map(|_|())
}
pub fn render(instance:&PluginInstance,now:u64,width:f32,height:f32)->Result<String,String>{
    let js=instance.definition.javascript.as_ref().ok_or("plugins-unavailable")?;
    let value=evaluate(&js.source,&serde_json::json!({"props":instance.values,"state":instance.state,"now":now,
        "elapsed_ms":now.saturating_sub(instance.created_at),"width":width.max(1.),"height":height.max(1.)}),
        "const canvas=new OmniCanvas(api.width,api.height);const output=plugin.render(api,canvas);return typeof output==='string'?output:canvas.toSVG();")?;
    let svg=value.as_str().ok_or("plugins-invalid-output")?;
    if !svg.trim_start().starts_with("<svg")||svg.len()>1_048_576{return Err("plugins-invalid-output".into());}
    Ok(svg.into())
}
pub fn dispatch(instance:&mut PluginInstance,event:&str,data:&Value)->Result<(),String>{
    let js=instance.definition.javascript.as_ref().ok_or("plugins-unavailable")?;
    if event.len()>128 || data.to_string().len()>65_536{return Err("plugins-invalid-output".into());}
    let value=evaluate(&js.source,&serde_json::json!({"props":instance.values,"state":instance.state,"now":crate::core::design::now_ms(),"event":{"type":event,"data":data}}),
        "if(typeof plugin.onEvent!=='function')return api.state;const result=plugin.onEvent(api,api.event);return result===undefined?api.state:result;")?;
    if value.to_string().len()>65_536{return Err("plugins-too-large".into());}
    instance.state=value.as_object().ok_or("plugins-invalid-output")?.clone();Ok(())
}
pub fn command(source:&str,props:&Map<String,Value>,context:&Value)->Result<Vec<super::runtime::HostAction>,String>{
    let result=evaluate(source,&serde_json::json!({"props":props,"context":context,"now":crate::core::design::now_ms()}),"return typeof plugin==='function'?plugin(api):plugin.run(api);")?;
    let actions:Vec<super::runtime::HostAction>=serde_json::from_value(result).map_err(|e|e.to_string())?;
    if actions.len()>32{return Err("plugins-invalid-output".into());}Ok(actions)
}
