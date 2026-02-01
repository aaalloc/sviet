struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

const EPSILON = 0.0001f;
const PI = 3.1415927f;
const FRAC_1_PI = 0.31830987f;
const FRAC_PI_2 = 1.5707964f;

const MIN_T = 0.001f;
const MAX_T = 1000f;

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> frame_data: Frame;
@group(0) @binding(2) var<uniform> render_param: RenderParam;
@group(0) @binding(3) var<storage, read_write> image_buffer: array<array<f32, 3>>;

@group(1) @binding(0) var<storage, read> objects: array<Object>;
@group(1) @binding(1) var<storage, read> spheres: array<Sphere>;
@group(1) @binding(2) var<storage, read> materials: array<Material>;
@group(1) @binding(3) var<storage, read> textures: array<array<f32, 3>>;
@group(1) @binding(4) var<storage, read> surfaces: array<Surface>;
@group(1) @binding(5) var<storage, read> lights: array<Light>;



@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(model.position, 0.0, 1.0),
        model.tex_coords,
    );
}

fn apply_transfer_function(x: f32) -> u32 {
    let a = 0.055;
    var y: f32;
    let xc = clamp(x, 0.0, 1.0);
    if xc > 0.0031308 {
        y = (1.0 + a) * pow(xc, 1.0 / 2.4) - a;
    } else {
        y = 12.92 * xc;
    }
    return u32(round(y * 255.0));
}
fn from_linear_rgb(c: vec3<f32>) -> vec3<f32> {

    let r = apply_transfer_function(c.x);
    let g = apply_transfer_function(c.y);
    let b = apply_transfer_function(c.z);

    return vec3<f32>(f32(r), f32(g), f32(b)) / 255.0;
}

// for webgpu
@fragment
fn fs_main_rgb(in: VertexOutput) -> @location(0) vec4<f32> {
    // Clamp to avoid the last pixel hitting exactly `width`/`height` due to interpolation.
    let u = clamp(in.tex_coords.x, 0.0, 0.99999994);
    let v = clamp(in.tex_coords.y, 0.0, 0.99999994);

    let x = min(u32(u * f32(frame_data.width)), frame_data.width - 1u);
    let y = min(u32(v * f32(frame_data.height)), frame_data.height - 1u);
    let i = y * frame_data.width + x;

    var rngState: u32 = init_rng(
        vec2<u32>(u32(x), u32(y)),
        vec2<u32>(frame_data.width, frame_data.height),
        frame_data.frame_idx
    );

    // Accumulate in linear space in the storage buffer.
    var pixel = vec3(image_buffer[i][0], image_buffer[i][1], image_buffer[i][2]);

    if render_param.clear_samples == 1u {
        pixel = vec3(0.0);
    }

    let rgb = sample_pixel(&rngState, f32(x), f32(y));
    pixel += rgb;
    image_buffer[i] = array<f32, 3>(pixel.r, pixel.g, pixel.b);

    // Avoid divide-by-zero if uniforms are ever out of sync.
    let denom = max(1.0, f32(render_param.total_samples));
    let linear_out = pixel / denom;
    let srgb_out = from_linear_rgb(linear_out);
    return vec4<f32>(srgb_out, 1.0);

    // var noiseState: u32 = init_rng(vec2<u32>(u32(u), u32(v)), vec2<u32>(512u, 512u), 0u);
    // return vec4<f32>(rng_next_float(&rngState), rng_next_float(&rngState), rng_next_float(&rngState), 1.0);
}

@fragment
fn fs_main_srgb(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.tex_coords.x;
    let v = in.tex_coords.y;

    let x = u32(u * f32(frame_data.width));
    let y = u32(v * f32(frame_data.height));
    let i = y * frame_data.width + x;

    var rngState: u32 = init_rng(
        vec2<u32>(u32(x), u32(y)),
        vec2<u32>(frame_data.width, frame_data.height),
        frame_data.frame_idx
    );

    var pixel = vec3(image_buffer[i][0], image_buffer[i][1], image_buffer[i][2]);

    if render_param.clear_samples == 1u {
        pixel = vec3(0.0);
    }

    let rgb = sample_pixel(&rngState, f32(x), f32(y));
    pixel += rgb;
    image_buffer[i] = array<f32, 3>(pixel.r, pixel.g, pixel.b);

    return vec4<f32>(
        pixel / f32(render_param.total_samples),
        1.0
    );

    // var noiseState: u32 = init_rng(vec2<u32>(u32(u), u32(v)), vec2<u32>(512u, 512u), 0u);
    // return vec4<f32>(rng_next_float(&rngState), rng_next_float(&rngState), rng_next_float(&rngState), 1.0);
}

struct RenderParam {
    samples_max_per_pixel: u32,
    samples_per_pixel: u32,
    total_samples: u32,
    clear_samples: u32,
    max_depth: u32,
};

struct Frame {
    width: u32,
    height: u32,
    frame_idx: u32,
};

struct Camera {
    eye: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    u: vec3<f32>,
    v: vec3<f32>,
    lensRadius: f32,
    lowerLeftCorner: vec3<f32>,
}

struct Object {
    id: u32,
    obj_type: u32,
    // for when object is has multiple meshes
    count: u32,
    offset: u32,
};

const OBJECT_SPHERE = 0u;
const OBJECT_MESHES = 1u;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct Sphere {
    center: vec4<f32>,
    radius: f32,
    material_index: u32,
};

struct Surface {
    vertices: array<vec4<f32>, 3>,
    normals: array<vec4<f32>, 3>,
};

const MAT_LAMBERTIAN = 0u;
const MAT_METAL = 1u;
const MAT_DIELECTRIC = 2u;
const MAT_DIFFUSE_LIGHT = 3u;

struct Material {
    id: u32,
    desc: TextureDescriptor,
    fuzz: f32,
};

struct TextureDescriptor {
    width: u32,
    height: u32,
    offset: u32,
}

struct HitRecord {
    p: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    material_index: u32,
    front_face: bool,
};


struct Scatter {
    ray: Ray,
    attenuation: vec3<f32>,
    type_pdf: u32,
}

struct Light {
    // index of the object
    id: u32,
    // sphere or mesh
    light_type: u32,
}

const PDF_NONE = 0u;
const PDF_COSINE = 1u;

fn hit_sphere(
    sphere_index: u32,
    material_index: u32,
    ray: Ray,
    ray_min: f32,
    ray_max: f32,
    hit: ptr<function, HitRecord>,
) -> bool {
    let sphere = spheres[sphere_index];

    let oc = ray.origin - sphere.center.xyz;
    let a = dot(ray.direction, ray.direction);
    let b = dot(ray.direction, oc);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - a * c;

    if discriminant < 0.0 {
        return false;
    }

    let sqrtd = sqrt(discriminant);

    var root = (-b - sqrtd) / a;
    if root < ray_min || root > ray_max {
        root = (-b + sqrtd) / a;
        if root < ray_min || root > ray_max {
            return false;
        }
    }


    *hit = sphereIntersection(ray, sphere, root, material_index);
    return true;
}

fn sphereIntersection(ray: Ray, sphere: Sphere, t: f32, material_index: u32) -> HitRecord {
    let p = ray.origin + t * ray.direction;
    var normal = (p - sphere.center.xyz) / sphere.radius;
    var front_face = true;
    if dot(ray.direction, normal) > 0.0 {
        normal = -normal;
        front_face = false;
    }
    return HitRecord(p, normal, t, material_index, front_face);
}

fn hit_triangle(
    triangle_index: u32,
    material_index: u32,
    ray: Ray,
    ray_min: f32,
    ray_max: f32,
    hit: ptr<function, HitRecord>,
) -> bool {
    let surface = surfaces[triangle_index];

    let e1 = surface.vertices[1].xyz - surface.vertices[0].xyz;
    let e2 = surface.vertices[2].xyz - surface.vertices[0].xyz;
    let h = cross(ray.direction, e2);
    let a = dot(e1, h);

    if a > -EPSILON && a < EPSILON {
        return false;
    }

    let f = 1.0 / a;
    let s = ray.origin - surface.vertices[0].xyz;
    let u = f * dot(s, h);

    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = cross(s, e1);
    let v = f * dot(ray.direction, q);

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * dot(e2, q);
    if t > ray_min && t < ray_max {
        let p = ray.origin + t * ray.direction;
        let b = vec3(1.0 - u - v, u, v);
        let n = b.x * surface.normals[0].xyz + b.y * surface.normals[1].xyz + b.z * surface.normals[2].xyz;
        let front_face = dot(ray.direction, n) < 0.0;
        *hit = HitRecord(p, normalize(n), t, material_index, front_face);
        return true;
    }

    return false;
}

fn check_intersection(ray: Ray, intersection: ptr<function, HitRecord>) -> bool {
    var closest_so_far = MAX_T;
    var hit_anything = false;
    var tmp_rec = HitRecord();

    for (var i = 0u; i < arrayLength(&objects); i += 1u) {
        let obj = objects[i];
        if obj.count > 1u {
            for (var j = 0u; j < obj.count; j += 1u) {
                if hit_triangle(obj.offset + j, obj.id, ray, MIN_T, closest_so_far, &tmp_rec) {
                    hit_anything = true;
                    closest_so_far = tmp_rec.t;
                    *intersection = tmp_rec;
                }
            }
        } else {
            if hit_sphere(obj.offset, i, ray, MIN_T, closest_so_far, &tmp_rec) {
                hit_anything = true;
                closest_so_far = tmp_rec.t;
                *intersection = tmp_rec;
            }
        }
    }
    return hit_anything;
}

fn sample_pixel(rngState: ptr<function, u32>, x: f32, y: f32) -> vec3<f32> {
    var color = vec3(0.0);
    for (var i = 0u; i < render_param.samples_per_pixel; i += 1u) {
        let ray = get_ray(rngState, x, y);
        color += ray_color(ray, rngState);
    }
    return color;
}

fn get_ray(rngState: ptr<function, u32>, x: f32, y: f32) -> Ray {
    let u = f32(x + rng_next_float(rngState)) / f32(frame_data.width);
    let v = f32(y + rng_next_float(rngState)) / f32(frame_data.height);

    let rd = camera.lensRadius * rng_in_unit_disk(rngState);

    let origin = camera.eye + rd.x * camera.u + rd.y * camera.v;
    let direction = camera.lowerLeftCorner + u * camera.horizontal + v * camera.vertical - origin;

    return Ray(origin, direction);
}


fn ray_color(first_ray: Ray, rngState: ptr<function, u32>) -> vec3<f32> {
    var ray = first_ray;
    var sky_color = vec3(0.0);
    var color_from_scatter = vec3(1.0);
    var color_from_emission = vec3(0.0);

    for (var i = 0u; i < render_param.max_depth; i += 1u) {
        var intersection = HitRecord();
        if !check_intersection(ray, &intersection) {
            let direction = normalize(ray.direction);
            let a = 0.5 * (direction.y + 1.0);
            // sky_color = (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1);
            break;
        }
        // for triangles only
        // if !intersection.front_face {
        //     continue;
        // }

        let material = materials[intersection.material_index];
        color_from_emission += color_from_scatter * emitted(material, 0.5, 0.5, intersection);

        var scattered = Scatter();
        if !scatter(&scattered, ray, intersection, material, rngState) {
            break;
        }
        if scattered.type_pdf == PDF_NONE {
            color_from_scatter *= scattered.attenuation;
            ray = scattered.ray;
            continue;
        }
        // scattered.ray = Ray(intersection.p, pdf_generate(rngState, intersection));

        // scattered.ray.direction = pdf_cosine_generate(rngState, pixar_onb(intersection.normal));
        // let pdf = pdf_cosine_value(scattered.ray.direction, pixar_onb(intersection.normal));

        // scattered.ray.direction = pdf_light_generate(rngState, intersection.p);
        // let pdf = pdf_light_value(intersection.p, scattered.ray.direction);

        // let pdf = pdf_mixed_value(
        //     pdf_cosine_value(scattered.ray.direction, pixar_onb(intersection.normal)),
        //     pdf_light_value(intersection.p, scattered.ray.direction)
        // );
        // let pdf1 = pdf_cosine_value(scattered.ray.direction, pixar_onb(intersection.normal));
        // let pdf2 = pdf_light_value(intersection.p, scattered.ray.direction);
        // let pdf = (0.5 * pdf1) + (0.5 * pdf2);
        
        // Use Mixed Sampling (MIS)
        let dir = pdf_generate(rngState, intersection);
        scattered.ray = Ray(intersection.p, dir);
        
        let pdf = pdf_mixed_value(
            pdf_cosine_value(scattered.ray.direction, pixar_onb(intersection.normal)),
            pdf_light_value(intersection.p, scattered.ray.direction)
        );

        let scattering_pdf = scattering_pdf_lambertian(intersection.normal, scattered.ray.direction);

        if pdf > 1e-6 {
             color_from_scatter *= (scattered.attenuation * scattering_pdf) / pdf;
        } else {
             color_from_scatter = vec3(0.0);
        }
        ray = scattered.ray;
    }
    return color_from_emission + color_from_scatter * sky_color;
}



struct ONB {
    u: vec3<f32>,
    v: vec3<f32>,
    w: vec3<f32>,
}

fn pixar_onb(n: vec3<f32>) -> ONB {
    // https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3<f32>(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3<f32>(b, s + n.y * n.y * a, -n.y);

    return ONB(u, v, n);
}

fn emitted(material: Material, u: f32, v: f32, hit: HitRecord) -> vec3<f32> {
    switch (material.id) {
        case MAT_DIFFUSE_LIGHT: {
            if hit.front_face {
                return texture_look_up(material.desc, u, v);
            } else {
                return vec3(0.0);
            }
        }
        default: {
            return vec3(0.0);
        }
    }
}

fn scatter(
    s: ptr<function, Scatter>,
    ray: Ray,
    hit: HitRecord,
    material: Material,
    rngState: ptr<function, u32>,
) -> bool {
    switch (material.id) 
    {
        case MAT_LAMBERTIAN:
        {
            (*s).attenuation = texture_look_up(material.desc, 0.5, 0.5);
            (*s).type_pdf = PDF_COSINE;
        }
        case MAT_METAL: 
        {
            var reflected = reflect(ray.direction, hit.normal);
            let fuzz = material.fuzz;
            reflected = normalize(reflected) + fuzz * rng_in_unit_sphere(rngState);
            *s = Scatter(
                Ray(hit.p, reflected),
                texture_look_up(material.desc, 0.5, 0.5), PDF_NONE
            );
        }
        case MAT_DIELECTRIC: 
        {
            var ri: f32 = material.fuzz;
            // use select here
            if hit.front_face {
                ri = 1.0 / material.fuzz;
            }

            let unit_direction = normalize(ray.direction);
            let cos_theta = min(dot(-unit_direction, hit.normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

            var direction = vec3(0.0);
            let rnd_float = rng_next_float(rngState);
            if ri * sin_theta > 1.0 || reflectance(cos_theta, ri) > rnd_float {
                direction = reflect(unit_direction, hit.normal);
            } else {
                direction = refract(unit_direction, hit.normal, ri);
            }
            *s = Scatter(
                Ray(hit.p, direction),
                vec3(1.0), PDF_NONE
            );
        }
        case MAT_DIFFUSE_LIGHT: 
        {
            return false;
        }
        default: {
            return false;
        }
    }
    return true;
}

fn scattering_pdf_lambertian(normal: vec3<f32>, direction: vec3<f32>) -> f32 {
    let cos_theta = dot(normalize(direction), normal);
    return select(0.0, cos_theta / PI, cos_theta > 0.0);
}

fn vec3_near_zero(v: vec3<f32>) -> bool {
    let s = 1e-8;
    return (abs(v.x) < s) && (abs(v.y) < s) && (abs(v.z) < s);
}

fn reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - 2.0 * dot(v, n) * n;
}

fn refract(uv: vec3<f32>, n: vec3<f32>, etai_over_etat: f32) -> vec3<f32> {
    let cos_theta = dot(-uv, n);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -sqrt(abs(1.0 - length(r_out_perp) * length(r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    var r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}


fn jenkin_hash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}

fn init_rng(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkin_hash(frame);
    return jenkin_hash(seed);
}


fn rng_on_hemisphere(rngState: ptr<function, u32>, normal: vec3<f32>) -> vec3<f32> {
    let on_unit_sphere = rng_in_unit_sphere(rngState);
    if dot(on_unit_sphere, normal) > 0.0 {
        return on_unit_sphere;
    } else {
        return -on_unit_sphere;
    }
}

fn rng_in_cosine_hemisphere(rngState: ptr<function, u32>) -> vec3<f32> {
    let r1 = rng_next_float(rngState);
    var r2 = rng_next_float(rngState);

    let z = sqrt(1.0 - r2);
    let phi = 2.0 * PI * r1;
    r2 = sqrt(r2);
    let x = cos(phi) * r2;
    let y = sin(phi) * r2;
    return vec3(x, y, z);
}

fn rng_in_unit_sphere(state: ptr<function, u32>) -> vec3<f32> {
    // Generate three random numbers x,y,z using Gaussian distribution
    var x = rng_next_float_gauss(state);
    var y = rng_next_float_gauss(state);
    var z = rng_next_float_gauss(state);

    // Multiply each number by 1/sqrt(x²+y²+z²) (a.k.a. Normalise) .
    // case x=y=z=0 ?

    let length = sqrt(x * x + y * y + z * z);
    return vec3(x, y, z) / length;
}

fn rng_in_unit_disk(state: ptr<function, u32>) -> vec2<f32> {
    var x = rng_next_float(state);
    var y = rng_next_float(state);
    return vec2(2.0 * x - 1.0, 2.0 * y - 1.0);
}

fn rng_next_int(state: ptr<function, u32>) -> u32 {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let newState = *state * 747796405u + 2891336453u;
    *state = newState;
    let word = ((newState >> ((newState >> 28u) + 4u)) ^ newState) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rng_next_float_gauss(state: ptr<function, u32>) -> f32 {
    var x1 = rng_next_float(state);
    let x2 = rng_next_float(state);
    if (x1 < 1e-6) { x1 = 1e-6; }
    return sqrt(-2.0 * log(x1)) * cos(2.0 * PI * x2);
}

fn rng_next_float_bounded(state: ptr<function, u32>, min: f32, max: f32) -> f32 {
    return min + rng_next_float(state) * (max - min);
}

fn rng_next_float(state: ptr<function, u32>) -> f32 {
    let x = rng_next_int(state);
    return f32(*state) / f32(0xffffffffu);
}

fn rng_next_vec3_surface(
    state: ptr<function, u32>,
    vertice: array<vec4<f32>, 3>
) -> vec3<f32> {
    let u = rng_next_float(state);
    let v = rng_next_float(state);

    let v0 = vertice[0].xyz; // corner
    let v1 = vertice[1].xyz; // right
    let v2 = vertice[2].xyz; // up

    var point = v0 + u * (v1 - v0) + v * (v2 - v0);
    return point;
}

fn rnd_to_sphere(radius: f32, distance_squared: f32, state: ptr<function, u32>) -> vec3<f32> {
    let r1 = rng_next_float(state);
    let r2 = rng_next_float(state);
    let z = 1.0 + r2 * (sqrt(1.0 - radius * radius / distance_squared) - 1.0);

    let phi = 2.0 * PI * r1;
    let x = cos(phi) * sqrt(1.0 - z * z);
    let y = sin(phi) * sqrt(1.0 - z * z);

    return vec3(x, y, z);
}

fn area_surface(vertice: array<vec4<f32>, 3>) -> f32 {
    let v0 = vertice[0].xyz; // corner
    let v1 = vertice[1].xyz; // right
    let v2 = vertice[2].xyz; // up

    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let c = cross(e1, e2);
    // return (c.x * c.x + c.y * c.y + c.z * c.z) * 0.5;
    return length(c) * 0.5;
}

fn pdf_sphere_value() -> f32 {
    return 1 / (4 * PI);
}

fn pdf_sphere_generate(state: ptr<function, u32>) -> vec3<f32> {
    return rng_in_unit_sphere(state);
}

fn pdf_cosine_value(direction: vec3<f32>, onb: ONB) -> f32 {
    let cosine_theta = dot(normalize(direction), onb.w);
    return max(cosine_theta / PI, EPSILON);
}

fn pdf_cosine_generate(state: ptr<function, u32>, onb: ONB) -> vec3<f32> {
    let rnd_direction = rng_in_cosine_hemisphere(state);
    return onb.u * rnd_direction.x + onb.v * rnd_direction.y + onb.w * rnd_direction.z;
}

fn pdf_light_generate(state: ptr<function, u32>, origin: vec3<f32>) -> vec3<f32> {
    let light_count = arrayLength(&lights);
    if (light_count == 0u) { return vec3(0.0, 1.0, 0.0); }
    
    let light_idx = min(u32(rng_next_float(state) * f32(light_count)), light_count - 1u);
    let light = lights[light_idx];
    let obj = objects[light.id];

    switch obj.obj_type {
        case OBJECT_SPHERE: {
            let sphere = spheres[obj.offset];
            let direction = sphere.center.xyz - origin;
            let distance = length(direction);
            let onb = pixar_onb(direction);
            let rnd_direction = rnd_to_sphere(sphere.radius, distance * distance, state);
            return onb.u * rnd_direction.x + onb.v * rnd_direction.y + onb.w * rnd_direction.z;
        }
        case OBJECT_MESHES: {
            let triangle_idx = u32(rng_next_float(state) * f32(obj.count));
            let vertices = surfaces[obj.offset + triangle_idx].vertices;
            let p = rng_next_vec3_surface(state, vertices);
            return normalize(p - origin);
        }
        default: {
            return vec3(0.0, 1.0, 0.0);
        }
    }
}

fn get_pdf_for_light(light_idx: u32, origin: vec3<f32>, direction: vec3<f32>) -> f32 {
    let light = lights[light_idx];
    let obj = objects[light.id];
    
    var hit = HitRecord();
    var hit_something = false;
    var closest = MAX_T;

    if (obj.obj_type == OBJECT_SPHERE) {
         if (hit_sphere(obj.offset, light.id, Ray(origin, direction), MIN_T, MAX_T, &hit)) {
             hit_something = true;
         }
    } else {
         var tmp_rec = HitRecord();
         for (var j = 0u; j < obj.count; j += 1u) {
            if (hit_triangle(obj.offset + j, light.id, Ray(origin, direction), MIN_T, closest, &tmp_rec)) {
                hit_something = true;
                closest = tmp_rec.t;
                hit = tmp_rec;
            }
         }
    }

    if (!hit_something) { return 0.0; }

    switch obj.obj_type {
        case OBJECT_SPHERE: {
            let sphere = spheres[obj.offset];
            let center_to_origin = origin - sphere.center.xyz;
            let dist_sq = dot(center_to_origin, center_to_origin); 

            let cos_theta_max = sqrt(1.0 - sphere.radius * sphere.radius / dist_sq);
            let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

            return 1.0 / solid_angle;
        }
        case OBJECT_MESHES: {
            // Approximation: Uses area of first triangle * count. 
            // Correct for quads/uniform meshes.
            let vertices = surfaces[obj.offset].vertices;
            let area = area_surface(vertices) * f32(obj.count);

            let dist_sq = hit.t * hit.t * dot(direction, direction);
            let cosine = abs(dot(direction, hit.normal) / length(direction));
            
            if (cosine < 1e-6) { return 0.0; }
            let pdf = dist_sq / (cosine * area);
            return pdf;
        }
        default: {
            return 0.0;
        }
    }
}

fn pdf_light_value(origin: vec3<f32>, direction: vec3<f32>) -> f32 {
    let light_len = arrayLength(&lights);
    if (light_len == 0u) { return 0.0; }

    var sum_pdf = 0.0;
    for (var i = 0u; i < light_len; i += 1u) {
        sum_pdf += get_pdf_for_light(i, origin, direction);
    }

    return sum_pdf / f32(light_len);
}

fn pdf_mixed_value(value1: f32, value2: f32) -> f32 {
    return (0.5 * value1) + (0.5 * value2);
}


fn pdf_generate(
    rngState: ptr<function, u32>,
    hit: HitRecord,
) -> vec3<f32> {
    if rng_next_float(rngState) < 0.5 {
        return pdf_cosine_generate(rngState, pixar_onb(hit.normal));
    } else {
        return pdf_light_generate(rngState, hit.p);
    }
}

fn pdf_value(pdf_type: u32, direction: vec3<f32>, onb: ONB) -> f32 {
    switch (pdf_type) {
        case PDF_NONE: {
            // should not happen
            return 0.0;
        }
        case PDF_COSINE: {
            return pdf_cosine_value(direction, onb);
        }
        default: {
            return 0.0;
        }
    }
}

fn texture_look_up(desc: TextureDescriptor, x: f32, y: f32) -> vec3<f32> {
    var u = clamp(x, 0f, 1f);
    var v = 1f - clamp(y, 0f, 1f);

    let j = u32(u * f32(desc.width));
    let i = u32(v * f32(desc.height));
    let idx = i * desc.width + j;

    let elem = textures[desc.offset + idx];
    return vec3(elem[0u], elem[1u], elem[2u]);
}
