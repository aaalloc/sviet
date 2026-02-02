# sviet

Yet another pathracer
Built with wgpu, winit, egui

## Status

Currently in work in progress

Implemented :
- [Raytracer the rest of your life](https://raytracing.github.io/books/RayTracingTheRestOfYourLife.html)
- BVH is also implemeted
- Model loading

![prev](image/raytracer_oneweekend.png)
![](image/cornell_box_suzanne.png)
![](image/raytracer_oneweekend_night.png)
<img width="2838" height="1734" alt="image" src="https://github.com/user-attachments/assets/bba566c7-6d66-48a3-8382-03cb2732db03" />


## WASM
web demo : https://yanovskyy.com/wasm/sviet

### dev
```
cargo install trunk
trunk build --features webgpu
trunk serve --features webgpu
```

### compile
```
wasm-pack build -t web --no-typescript --release --out-name sviet
```

