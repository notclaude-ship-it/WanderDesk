mod catalog;
mod settings;

use bevy::{input::mouse::MouseMotion, prelude::*, window::{CursorGrabMode, PrimaryWindow}};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use catalog::{discover_apps, launch_app, AppEntry};
use settings::Settings;

#[derive(Resource)]
struct DesktopState {
    apps: Vec<AppEntry>, settings: Settings, menu: bool, search: String,
    notice: String, yaw: f32, pitch: f32,
}
#[derive(Component)] struct PlayerCamera;

fn main() {
    let settings = Settings::load();
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.008, 0.016, 0.04)))
        .insert_resource(DesktopState { apps: discover_apps(), settings, menu: true, search: String::new(), notice: "ZQSD/WASD : bouger · souris : regarder · E : ouvrir le portail".into(), yaw: 0.0, pitch: 0.0 })
        .add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { title: "WanderDesk — 3D Desktop World".into(), resolution: (1440.0_f32, 900.0_f32).into(), ..default() }), ..default() }))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup_world)
        .add_systems(Update, (desktop_ui, player_controls, portal_interaction))
        .run();
}

fn setup_world(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, state: Res<DesktopState>) {
    commands.spawn((Camera3dBundle { transform: Transform::from_xyz(0.0, 1.8, 9.0).looking_at(Vec3::new(0.0, 1.5, -2.0), Vec3::Y), ..default() }, PlayerCamera));
    commands.insert_resource(AmbientLight { color: Color::rgb(0.32, 0.48, 0.72), brightness: 0.7 });
    commands.spawn(DirectionalLightBundle { directional_light: DirectionalLight { illuminance: 14000.0, shadows_enabled: false, ..default() }, transform: Transform::from_xyz(-5.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y), ..default() });
    let floor = materials.add(StandardMaterial { base_color: Color::rgb(0.018,0.035,0.065), metallic: 0.45, perceptual_roughness: 0.4, ..default() });
    commands.spawn(PbrBundle { mesh: meshes.add(Mesh::from(shape::Plane { size: 180.0, subdivisions: 10 })), material: floor, ..default() });
    let grid = materials.add(StandardMaterial { base_color: Color::rgb(0.0,0.36,0.52), emissive: Color::rgb(0.0,0.08,0.13), metallic: 0.6, ..default() });
    for n in -35..=35 {
        let p=n as f32*2.0;
        commands.spawn(PbrBundle { mesh: meshes.add(Mesh::from(shape::Box::new(0.018,0.012,140.0))), material:grid.clone(), transform:Transform::from_xyz(p,0.012,0.0), ..default() });
        commands.spawn(PbrBundle { mesh: meshes.add(Mesh::from(shape::Box::new(140.0,0.012,0.018))), material:grid.clone(), transform:Transform::from_xyz(0.0,0.013,p), ..default() });
    }
    let hub = materials.add(StandardMaterial { base_color:Color::rgb(0.035,0.075,0.13), metallic:0.82, perceptual_roughness:0.2, ..default() });
    commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::Cylinder { radius:5.8,height:0.22,resolution:96,segments:1 })), material:hub, transform:Transform::from_xyz(0.0,0.15,-3.0), ..default() });
    let glow = materials.add(StandardMaterial { base_color:Color::rgb(0.02,0.72,0.94), emissive:Color::rgb(0.0,0.25,0.4), metallic:0.5, ..default() });
    for n in 0..24 { let a=n as f32*std::f32::consts::TAU/24.0; commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::Box::new(0.1,0.8,0.1))), material:glow.clone(), transform:Transform::from_xyz(a.cos()*5.3,0.5,-3.0+a.sin()*5.3), ..default() }); }
    let colors=[Color::rgb(0.02,0.78,0.98),Color::rgb(0.58,0.24,1.0),Color::rgb(0.1,0.95,0.58),Color::rgb(1.0,0.38,0.12),Color::rgb(0.98,0.25,0.48),Color::rgb(0.95,0.8,0.18)];
    let count=state.apps.len().clamp(6,18);
    for i in 0..count {
        let a=i as f32*std::f32::consts::TAU/count as f32; let p=Vec3::new(a.cos()*4.1,1.7,-3.0+a.sin()*4.1); let c=colors[i%colors.len()];
        let frame=materials.add(StandardMaterial { base_color:c, emissive:c*0.75, metallic:0.4, perceptual_roughness:0.2, ..default() });
        let glass=materials.add(StandardMaterial { base_color:Color::rgb(0.02,0.06,0.12), metallic:0.4, ..default() });
        let rotation=Transform::from_translation(p).looking_at(Vec3::new(0.0,1.5,-3.0),Vec3::Y).rotation;
        commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::Box::new(1.3,2.25,0.16))), material:frame.clone(), transform:Transform { translation:p, rotation, ..default() }, ..default() });
        commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::Box::new(1.12,2.05,0.18))), material:glass, transform:Transform { translation:p+Vec3::new(0.0,0.0,0.02), rotation, ..default() }, ..default() });
        commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::UVSphere { radius:0.28,sectors:24,stacks:16 })), material:frame, transform:Transform::from_translation(p+Vec3::new(0.0,0.2,0.14)), ..default() });
        if let Some(app) = state.apps.get(i) {
            commands.spawn(Text2dBundle { text: Text::from_section(app.name.clone(), TextStyle { font_size: 22.0, color: Color::WHITE, ..default() }), transform: Transform { translation:p+Vec3::new(0.0,-0.62,0.18), rotation, ..default() }, ..default() });
        }

    }
    let dark=materials.add(StandardMaterial { base_color:Color::rgb(0.035,0.06,0.1), metallic:0.75, perceptual_roughness:0.28, ..default() });
    for (x,z,h) in [(-9.0,-8.0,7.0),(9.0,-8.0,7.0),(-9.0,3.0,4.5),(9.0,3.0,4.5),(-6.5,-13.0,5.0),(6.5,-13.0,5.0)] { commands.spawn(PbrBundle { mesh:meshes.add(Mesh::from(shape::Box::new(0.8,h,0.8))), material:dark.clone(), transform:Transform::from_xyz(x,h/2.0,z), ..default() }); }
}

fn player_controls(time:Res<Time>, keys:Res<Input<KeyCode>>, mut mouse:EventReader<MouseMotion>, buttons:Res<Input<MouseButton>>, mut windows:Query<&mut Window,With<PrimaryWindow>>, mut state:ResMut<DesktopState>, mut camera:Query<&mut Transform,With<PlayerCamera>>) {
    let Ok(mut t)=camera.get_single_mut() else{return}; if state.menu { mouse.clear(); return; }
    if let Ok(mut w)=windows.get_single_mut() { if buttons.just_pressed(MouseButton::Left) { w.cursor.grab_mode=CursorGrabMode::Locked; w.cursor.visible=false; } if keys.just_pressed(KeyCode::Escape) { w.cursor.grab_mode=CursorGrabMode::None; w.cursor.visible=true; state.menu=true; } }
    let mut m=Vec3::ZERO; let f=t.forward(); let r=t.right();
    if keys.pressed(KeyCode::W)||keys.pressed(KeyCode::Z){m+=f;} if keys.pressed(KeyCode::S){m-=f;} if keys.pressed(KeyCode::A)||keys.pressed(KeyCode::Q){m-=r;} if keys.pressed(KeyCode::D){m+=r;} if keys.pressed(KeyCode::Space){m.y+=1.0;} if keys.pressed(KeyCode::ShiftLeft){m.y-=1.0;}
    if m.length_squared()>0.0 { t.translation+=m.normalize()*(if keys.pressed(KeyCode::ControlLeft){13.0}else{6.0})*time.delta_seconds(); }
    if buttons.pressed(MouseButton::Left) { for ev in mouse.read(){state.yaw-=ev.delta.x*0.0025;state.pitch=(state.pitch-ev.delta.y*0.0025).clamp(-1.45,1.45);} t.rotation=Quat::from_rotation_y(state.yaw)*Quat::from_rotation_x(state.pitch); } else { mouse.clear(); }
}

fn portal_interaction(keys:Res<Input<KeyCode>>,mut state:ResMut<DesktopState>,camera:Query<&Transform,With<PlayerCamera>>) {
    if !keys.just_pressed(KeyCode::E){return} let Ok(t)=camera.get_single() else{return};
    let count=state.apps.len().clamp(6,18); let mut nearest=None;
    for i in 0..count { let a=i as f32*std::f32::consts::TAU/count as f32; let p=Vec3::new(a.cos()*4.1,1.7,-3.0+a.sin()*4.1); let d=t.translation.distance(p);
        if d<3.6&&t.forward().dot((p-t.translation).normalize())>0.35&&nearest.map(|n:(f32,usize)|d<n.0).unwrap_or(true){nearest=Some((d,i));}
    }
    if let Some((_,i))=nearest { if let Some(app)=state.apps.get(i){state.notice=match launch_app(app){Ok(())=>format!("Lancement : {}",app.name),Err(e)=>e};} } else {state.notice="Approche-toi d'un portail et regarde-le, puis appuie sur E.".into();}
}

fn desktop_ui(mut contexts:EguiContexts,mut state:ResMut<DesktopState>,mut windows:Query<&mut Window,With<PrimaryWindow>>) {
    let ctx=contexts.ctx_mut();
    egui::TopBottomPanel::top("hud_top").frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(4,10,24,220))).show(ctx,|ui|{ui.horizontal(|ui|{ui.heading(egui::RichText::new("WANDERDESK // 3D WORLD").strong().color(egui::Color32::from_rgb(65,225,255)));ui.separator();ui.label(format!("{} applications",state.apps.len()));ui.separator();ui.label("ZQSD/WASD · souris · E ouvrir · Échap menu");if ui.button("HUB").clicked(){state.menu=true;}});});
    egui::TopBottomPanel::bottom("hud_bottom").frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(4,10,24,220))).show(ctx,|ui|{ui.horizontal(|ui|{ui.label(egui::RichText::new("● GPU PBR RENDERER").color(egui::Color32::from_rgb(45,245,170)));ui.separator();ui.label(&state.notice);});});
    if state.menu { egui::Window::new("WANDERDESK / CONTROL CENTER").default_size([540.0,570.0]).resizable(true).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0]).show(ctx,|ui|{
        ui.heading(egui::RichText::new("Explore ton bureau en 3D").size(25.0).color(egui::Color32::from_rgb(65,225,255)));
        ui.label("Scène 3D native : profondeur, éclairage, matériaux métalliques, reflets PBR et portails d'applications.");ui.separator();
        ui.horizontal(|ui|{ui.label("Vitesse");ui.add(egui::Slider::new(&mut state.settings.speed,1.0..=15.0));});
        if ui.button("▶ ENTRER DANS LE MONDE").clicked(){state.menu=false;if let Ok(mut w)=windows.get_single_mut(){w.cursor.grab_mode=CursorGrabMode::None;w.cursor.visible=true;}}
        if ui.button("↻ Actualiser les applications").clicked(){state.apps=discover_apps();}
        ui.separator();ui.horizontal(|ui|{ui.label("Recherche");ui.text_edit_singleline(&mut state.search);});
        egui::ScrollArea::vertical().max_height(290.0).show(ui,|ui|{let q=state.search.to_lowercase();let items:Vec<(usize,String)>=state.apps.iter().enumerate().filter(|(_,a)|q.is_empty()||a.name.to_lowercase().contains(&q)).map(|(i,a)|(i,a.name.clone())).collect();for (i,name) in items{ui.horizontal(|ui|{ui.label("◆");ui.label(name.clone());if ui.button("Lancer").clicked(){state.notice=match launch_app(&state.apps[i]){Ok(())=>format!("Lancement : {name}"),Err(e)=>e};}});}});
        ui.separator();ui.label("Souris : regarder · ZQSD/WASD : marcher · Espace/Maj : monter/descendre · Ctrl : accélérer · E : lancer le portail regardé · Échap : menu");
    });}
}