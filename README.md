# RT Rust

RT Rust is a path tracer written in Rust. The aim of RT Rust is to
create a powerful and flexible path tracer along with a artist
friendly UI to render scenes of any complexity.

Project name is still a todo :P

Sections:

* [Features](#features)
* [Screenshots](#screenshots)
* [Compilation](#compilation)
* [Renders](#renders)
* [TODOs](#todos)

## Features

* Unbiased path tracer

* Artist friendly viewport and GUI

* Camera with real world parameters

* BSDFs
  * Lambert
  * Glossy
  * Glass
  * Emission
  * Refraction
  * Blinnphong

* Viewport rendering
  * Interactive path tracing in the viewport

* Graphical User Interface (GUI)
  * Wavefront OBJ mesh loading
  * Shader assignment
  * Texture loading
  * Camera and other render settings
  * Scene save to and load from disk

* Command Line Interface (CLI)
  * Partial scene setup

* Headless mode
  * Progress bar
  * Stop at any sample

* High dynamic range environment map support

* Accelerated using [Embree](https://www.embree.org/) by introducing a
  [safe Rust wrapper](https://github.com/ishbosamiya/embree_rust/)

* Progressive rendering

* Ray path visualization

* Spectral rendering support
  * See
  [spectral_rendering](https://github.com/ishbosamiya/rt_rust/tree/spectral_rendering)
  branch

## Screenshots

#### GUI
![GUI](/screenshots/rt_rust_gui.png)

#### Viewport Rendering
https://user-images.githubusercontent.com/17758946/145869987-a1ce3ebd-276c-4050-b564-47eadc87e30e.mp4

## Compilation

### Dependencies

GLFW3, Threading Building Blocks (TBB), OpenGL

#### Linux (Debian)

```bash
sudo apt install libglfw3-dev libtbb-dev
```

### Compile and run

```bash
git clone https://github.com/ishbosamiya/rt_rust.git
cd rt_rust
cargo run --release
```

For Linux, it should utilize precompiled static libraries for Embree
so it shouldn't take too long to compile. For Windows and MacOS, no
precompiled static libraries for Embree are available yet so first
compile will take a long time, it compiles Embree from scratch.

#### Test scenes

```bash
git submodule init
git submodule update
```

##### Render test scenes
Renders the test scenes to their respective output folders. Note that
this will take a long time, the test scenes are supposed to be
rendered for debugging purposes. It is recommended to run this only if
you are an active developer for the project. The final renders can be
found [here](https://github.com/ishbosamiya/rt_rust_test_scenes/) for
anyone else.

```bash
cargo build --release && cargo run -p testing_framework -- `cat ./test_scenes/car_cli_args.txt`
cargo build --release && cargo run -p testing_framework -- `cat ./test_scenes/cornell_box_cli_args.txt`
cargo build --release && cargo run -p testing_framework -- `cat ./test_scenes/materialball_cli_args.txt`
```

##### Load test scene in GUI
To load any test scene in the GUI run any [render test
scenes](#render-test-scenes) command with `--dry-run` appended to the
end. Copy the arguments that should be passed to the main binary. Note
that some test scenes might have a different working directory, so the
current directory should be set to that directory if needed. See the
[example](#example-test-scene-in-gui) to understand better.

###### Example test scene in GUI
Considering the car scene as an example

```bash
cargo build --release && cargo run -p testing_framework -- `cat ./test_scenes/car_cli_args.txt` --dry-run
```

Copy the arguments after `--headless` to before `--output` and append
to `cargo run --release -- ` to form the complete command. It is
important to change the current directory to the current working
directory mentioned in the CLI args. For this example, must change the
current directory to the `test_scenes` directory.

```bash
cd test_scenes
cargo run --release -- --threads 0 --width 1920 --height 1080 --samples 5000 --trace-max-depth 50 --environment ./hdrs/studio_light_box.hdr --environment-strength 3 --obj-files ./car/gto67.obj ./car/gto67_ground_plane.obj --object-shader front_rim.001,metal_light --object-shader rear_view_mirror,tyre --object-shader back_keys,metal_light --object-shader under_body,black_plastic --object-shader door_handle,body_paint_black --object-shader backlight_rim,black_plastic --object-shader side_spoiler,body_paint_green --object-shader window_back,window --object-shader back_rim,black_plastic --object-shader windshield,window --object-shader front_rim,black_plastic --object-shader back_bumper,body_paint_green --object-shader window_rim.003,black_plastic --object-shader seats,leather --object-shader ground,ground --object-shader side_view_mirror_stem,body_paint_black --object-shader inner_panel,black_plastic --object-shader front_bumper,body_paint_green --object-shader windshield_rim,black_plastic --object-shader window_rim.001,black_plastic --object-shader front_spoiler,black_plastic --object-shader tyre_rear_Cylinder.003,tyre --object-shader steering_wheel_spokes,black_plastic --object-shader light_front,light --object-shader steer_wheel_column,black_plastic --object-shader backlight,black_plastic --object-shader front_grill,metal_light --object-shader door_inner,black_plastic --object-shader window_rim.002,tyre --object-shader window_rim.004,black_plastic --object-shader wheel_Cylinder.004,metal_light --object-shader window,window --object-shader steering_wheel,black_plastic --object-shader rim_light,black_plastic --object-shader rim_side,black_plastic --object-shader glass_back,window --object-shader window_rim,black_plastic --object-shader inner_body,black_plastic --object-shader mud_guard,black_plastic --object-shader body,body_paint_green --object-shader wheel_rear_Cylinder.004,metal_light --object-shader tyres_Cylinder.003,tyre --object-shader back_plate,body_paint_black --object-shader dashboard,black_plastic --object-shader door,body_paint_green --object-shader bonnet,body_paint_black --object-shader hood,body_paint_black --object-shader glass_back_rim,black_plastic --object-shader inner,black_plastic --object-shader side_view_mirror,body_paint_black --rt-file ./car/gto67.rt
```

## Renders
#### Pontiac GTO 67 Render
Model Credits: [thecali from Blend Swap](https://www.blendswap.com/blend/13575)

![Car Render](/renders/gto67.png)

#### Pontiac GTO 67 Clay Render
Model Credits: [thecali from Blend Swap](https://www.blendswap.com/blend/13575)

![Car Render Clay](/renders/gto67_clay_render.png)

#### Cornell Box with Blender Monkey - Grey Lambert Shader
![Cornell Box Monkey Grey Lambert](/renders/cornell_box_monkey_grey_lambert.png)

#### Cornell Box with Blender Monkey - Glass Shader
![Cornell Box Monkey Glass](/renders/cornell_box_monkey_glass.png)

#### Materialball - Glass
Model Credits: [victorborges from Blend Swap](https://www.blendswap.com/blend/11511)

![Materialball Glass](/renders/glass.png)

#### Materialball - Diffuse
Model Credits: [victorborges from Blend Swap](https://www.blendswap.com/blend/11511)

![Materialball Diffuse](/renders/diffuse.png)

#### Materialball - Glossy
Model Credits: [victorborges from Blend Swap](https://www.blendswap.com/blend/11511)

![Materialball Glossy](/renders/glossy.png)

## TODOs

* [ ] Parse .blend files for Scene data
* [ ] Microfacet models for certain BSDFs
* [ ] Disney BSDF
* [ ] Light falloff support
* [ ] Shader nodes
* [ ] Importance sampling
* [ ] Improved RT file to reduce file size
