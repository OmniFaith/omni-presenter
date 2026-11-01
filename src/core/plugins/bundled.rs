use super::*;

pub fn bundled() -> Vec<InstalledPlugin> {
    [include_str!("../../../assets/plugins/stage-tools.json")]
    .into_iter()
    .map(|source| InstalledPlugin {
        package: serde_json::from_str(source).expect("invalid bundled plugin"),
        enabled: true,
        settings: Map::new(),
    })
    .collect()
}

pub fn migrate(plugins:&mut Vec<InstalledPlugin>){
    plugins.retain(|plugin|plugin.package.id!="omni-presenter.text-tools");
    let current=bundled().remove(0).package;
    for plugin in plugins {
        if plugin.package.id==current.id {
            plugin.package=current.clone();
        }
    }
}
pub fn migrate_instance(instance:&mut PluginInstance){
    let package=bundled().remove(0).package;
    if instance.plugin_id=="omni-presenter.stage-tools" && instance.version!=package.version
        && let Some(component)=package.components.iter().find(|component|component.id==instance.definition.id){
            instance.definition=component.clone();instance.version=package.version;
            for field in &component.fields{instance.values.entry(field.id.clone()).or_insert_with(||field.default.clone());}
        }
}
