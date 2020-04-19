extern crate serde_json;
extern crate ndarray;
extern crate pi3d;

use std::error::Error;
use std::env;
use std::collections::HashMap;
use ndarray as nd;
use ndarray::s;

const SZ: f32 = 5.0;
const REPEL: f32 = 0.06;
const ATTRACT: f32 = 0.02;
const RAD: f32 = 0.05;
const LIMIT: f32 = 10.0;
const W:f32 = 960.0; // screen
const H:f32 = 720.0;

// match using truth grid
fn match_flat(fr_flat: &Vec<usize>, to_check_flat: &Vec<u32>) -> bool {
    let mut fr_ref: Vec<Option<u32>> = vec![None; 20];
    let n = to_check_flat.len();
    for i in 0..n {
        match fr_ref[fr_flat[i]] {
            Some(v) => {
                if v != to_check_flat[i] {
                    return false;
                }
            },
            None => {
                fr_ref[fr_flat[i]] = Some(to_check_flat[i]);
            },
        }
    }
    true
}


fn recurse(edges: &Vec<Vec<u32>>, fr_flat: &Vec<usize>, fr: &Vec<Vec<u32>>,
        found_list: &mut Vec<usize>, found_flat: &mut Vec<u32>) -> bool {
    for i in 0..edges.len() {
        if !found_list.contains(&i) &&
                edges[i].len() == fr[found_list.len()].len() {
            let mut new_found_flat = found_flat.clone();
            new_found_flat.extend(&edges[i]);
            if match_flat(fr_flat, &new_found_flat) {
                found_list.push(i);
                found_flat.extend(&edges[i]);
                if found_list.len() == fr.len() { // match so far and right length
                    return true;
                } else { // match but more to do
                    if recurse(&edges, fr_flat, fr, found_list, found_flat) {
                        return true;
                    } // else continue to the next edge in edges
                } 
            } else { // if at end then go back one step
                if i == edges.len() - 1 { // last one
                    match found_list.pop() {
                        Some(x) => {
                            let n_to_remove = edges[x].len();
                            for _ in 0..n_to_remove {
                                found_flat.pop();
                            }
                        },
                        None => {},
                    }
                }
            }
        }
    }
    false
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    // rule-from     rule-to                   start-graph   num
    // [[0,1],[0,2]] [[1,3],[1,5],[2,5],[3,5]] [[0,1],[0,2]] 15
    let fr: Vec<Vec<u32>> = serde_json::from_str(&args[1])?;
    let to: Vec<Vec<u32>> = serde_json::from_str(&args[2])?;
    let num: usize = serde_json::from_str(&args[4])?;
    // use flat list for checking number patterns
    let fr_flat: Vec<usize> = fr.iter().flatten().map(|v| *v as usize).collect();
    // let fr_num = fr.len(); // number of edges needed to match a from pattern
    let mut edges: Vec<Vec<u32>> = serde_json::from_str(&args[3])?;
    edges.reserve(5000);
    let mut next_num = edges.iter().flatten().max().unwrap() + 1; // one more than max already used
    let mut out_edges = Vec::<Vec<Vec<u32>>>::with_capacity(5000);
    for _gen in 0..num {
        out_edges.clear();
        let mut found_one = true;
        while found_one {
            let mut found_list: Vec<usize> = vec![]; // skip over these while looking
            let mut found_flat: Vec<u32> = vec![];
            found_one = recurse(&edges, &fr_flat, &fr, &mut found_list, &mut found_flat);
            if found_one {
                let mut new_out_edge = vec![];
                for k in found_list.iter() {
                    new_out_edge.push(edges[*k].clone());
                }
                found_list.sort();
                for k in found_list.iter().rev() { // reverse order so as to not scramble
                    edges.remove(*k);
                }
                out_edges.push(new_out_edge);
            }
        }
        for edge in &out_edges {
            let mut newly_made: Vec<(u32, u32)> = vec![];
            for new_pattern in &to {
                let mut new_edge = vec![];
                for p in new_pattern {
                    let mut make_new = true;
                    'skip: for q in 0..fr.len() {
                        for r in 0..fr[q].len() {
                            if fr[q][r] == *p { // existing node not new one
                                new_edge.push(edge[q][r]);
                                make_new = false;
                                break 'skip;
                            }
                        }
                    }
                    if make_new {
                        for (q, r) in &newly_made {
                            if q == p { // one manufactured in this edge conversion
                                new_edge.push(*r);
                                make_new = false;
                                break;
                            }
                        }
                        if make_new { // could have been switched to false by re-using newly_made
                            new_edge.push(next_num);
                            newly_made.push((*p, next_num));
                            next_num += 1;
                        }
                    }
                }
                edges.push(new_edge);
            }
        }
        edges.sort();
    }
    // now pack them so there are no blanks, start at 0
    let mut pack = HashMap::<u32, u32>::with_capacity(5000);
    next_num = 0;
    for i in edges.iter().flatten() {
        if !pack.contains_key(&i) {
            pack.insert(*i, next_num);
            next_num += 1;
        }
    }
    for i in 0..edges.len() {
        for j in 0..edges[i].len() {
            edges[i][j] = pack[&edges[i][j]];
        }
    }
    // create nd::Array2<f32> for vertices. giving x,y,z base on index num
    let n_verts = next_num as usize;
    println!("\n{}", n_verts);
    let mut verts = nd::Array2::<f32>::zeros([n_verts, 3]);
    let mut vels = nd::Array2::<f32>::zeros([n_verts, 3]);
    for i in 0..n_verts { // approx uniform distribution
        verts[[i, 0]] = (i % 17) as f32 * SZ / 17.0 - 0.5 * SZ;
        verts[[i, 1]] = ((i + 5) % 7) as f32 * SZ / 7.0 - 0.5 * SZ;
        verts[[i, 2]] = ((i + 31) % 11) as f32 * SZ / 11.0 - 0.5 * SZ;
    }
    // loop
    for _n in 0..200 { // annealing process -> zero velocity
    // find c of g then dx,dy,dz -> 1/magnitude sq, direction force out
        let c_of_g = verts.mean_axis(nd::Axis(0)).unwrap();
        verts -= &c_of_g;
        let dist_sq = &verts * &verts + 0.00001; // to avoid div zero later
        let mag = &dist_sq.sum_axis(nd::Axis(1))
                    .mapv(f32::sqrt)
                    .into_shape((n_verts, 1)).unwrap();
        vels = vels + &verts * REPEL / mag.mapv(|v| v.powi(2)); // inverse linear
        for i in 0..edges.len() {
            let nv = edges[i].len();
            for j in 0..(nv-1) { // must be at least two nodes for an edge
                for k in (j+1)..nv {
                    let e_dist = &verts.slice(s![edges[i][k] as usize,..])
                               - &verts.slice(s![edges[i][j] as usize,..]);
                    let e_dist_sq = &e_dist * &e_dist + 0.00001;
                    let e_mag = &e_dist_sq.sum(); // scalar
                    let mut e_acc = e_dist * ATTRACT;// / e_mag.powi(3);
                    if e_mag < &RAD { // repel near to
                        e_acc *= -1.0;
                    }
                    let mut vel_slice = vels.slice_mut(s![edges[i][j] as usize,..]);
                    vel_slice += &e_acc;
                    vel_slice = vels.slice_mut(s![edges[i][k] as usize,..]);
                    vel_slice -= &e_acc;
                }
            }
        }
        vels *= 0.75; // friction
        verts += &vels; // displacement
        verts.mapv_inplace(|v| v.max(-LIMIT).min(LIMIT));
    // TODO tex coords based on spherical projection from c of g
    }
    let mut display = pi3d::display::create("ESC to quit", W, H, "GL", 2, 1).unwrap();
            display.set_background(&[0.05, 0.05, 0.1, 1.0]);
            display.set_target_fps(30.0);
    let shader = pi3d::shader::Program::from_res("uv_reflect").unwrap();
    let camera = pi3d::camera::create(&display);
    let tex = pi3d::texture::create_from_file("stars.jpg");
    // make lines using pi3d
    let mut norms = verts.clone(); // radially from c_of_g i.e. normalised positions
    let dist_sq = &verts * &verts + 0.00001; // to avoid div zero later
    let mag = &dist_sq.sum_axis(nd::Axis(1))
                .mapv(f32::sqrt)
                .into_shape((n_verts, 1)).unwrap();
    norms = norms / mag;
    let mut tex_coords = nd::Array2::<f32>::zeros((n_verts, 2));
    for i in 0..n_verts {
        tex_coords[[i, 0]] = verts[[i,2]].atan2(verts[[i,0]]) * 0.5 * std::f32::consts::FRAC_1_PI + 0.5;
        let radial = (verts[[i,0]].powi(2) + verts[[i,2]].powi(2)).sqrt();
        tex_coords[[i, 1]] = verts[[i,1]].atan2(radial) * std::f32::consts::FRAC_1_PI + 0.5;
    }
    let mut faces = Vec::<u16>::with_capacity(edges.len() * 2);
    for i in 0..edges.len() {
        let nv = edges[i].len();
        for j in 0..(nv-1) { // must be at least two
            faces.push(edges[i][j] as u16);
            faces.push(edges[i][j+1] as u16);
        }
    }
    while (faces.len() % 3) != 0 {
        faces.push(0);
    }
    let nfaces = faces.len() / 3;
    let mut new_buffer = pi3d::buffer::create(&shader, verts, norms, tex_coords,
        nd::Array::from_shape_vec((nfaces, 3usize), faces).unwrap(), false);
    new_buffer.set_line_width(4.0, false, false);
    let mut graph = pi3d::shape::create(vec![new_buffer], camera.reference());
    graph.position(&[0.0, 0.0, 10.0]);
    graph.set_fog(&[0.7, 0.7, 0.3], 14.8, 0.01);
    graph.set_material(&[0.6, 0.5, 1.0]);
    graph.set_textures(&vec![tex.id, tex.id, tex.id]);
    while display.loop_running() { // default sdl2 check for ESC or click cross
        graph.draw();
        graph.rotate_inc_y(0.01);
        graph.rotate_inc_x(0.001);
    }
    Ok(())
}
