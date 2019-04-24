#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include <string>
#include <iostream>
#include <vector>
#include <algorithm>

#include <thrust/host_vector.h>
#include <thrust/device_vector.h>
#include <thrust/transform_scan.h>
#include <thrust/scan.h>

typedef  int64_t i64;
typedef uint64_t u64;
typedef double   f64;

// ::::::::::
// :: Node ::
// ::::::::::

__host__ __device__
u64 new_node(u64 kind, i64 a_dist, u64 a_slot, i64 b_dist, u64 b_slot, i64 c_dist, u64 c_slot) {
  return (kind << 54)
      | (a_slot << 52) | ((u64)(a_dist + 32768) << 36)
      | (b_slot << 34) | ((u64)(b_dist + 32768) << 18)
      | (c_slot << 16) | ((u64)(c_dist + 32768) <<  0);
}

__host__ __device__
u64 get_kind(u64 node) {
  return (node >> 54) & 0xFF;
}

__host__ __device__
i64 get_dist(u64 node, u64 slot) {
  return (i64)((node >> (36 - slot * 18)) & 0xFFFF) - 32768;
}

__host__ __device__
u64 get_slot(u64 node, u64 slot) {
  return ((node >> (52 - slot * 18)) & 0x3);
}

__host__ __device__
u64 inc_port(u64 node, u64 slot, i64 delta) {
  return (u64)((i64)node + (delta << (36 - slot * 18)));
}

__host__ __device__
u64 inc_ports(u64 node, i64 delta) {
  return (u64)((i64)node + (delta << 36) + (delta << 18) + delta);
}

__host__ __device__
u64 set_port(u64 node, u64 slot, i64 new_dist, u64 new_slot) {
  return (node & ~((u64)0x3FFFF << (36 - slot * 18))) | (((new_slot << 16) | (u64)(new_dist + 32768)) << (36 - slot * 18));
}

__host__ __device__
u64 eql(u64 a, u64 b) {
  return a == b;
}

__host__ __device__
f64 get_force(u64 node) {
  i64 x = get_dist(node, 0);
  i64 y = get_dist(node, 1);
  i64 z = get_dist(node, 2);
  return (f64)((x < 0 ? -1 : 1) * x * x + (y < 0 ? -1 : 1) * y * y + (z < 0 ? -1 : 1) * z * z);
}

__constant__
const u64 air = 0x8000600028000; // new_node(0, 0,0, 0,1, 0,2)

// :::::::::
// :: Net ::
// :::::::::

struct Alloc {
  u64 indxs[4];
};

__host__ __device__
bool alloc4(u64 *net, u64 len, u64 i, u64 *indxs) {
  u64 k = 0, n, a;
  u64 j = 0;
  do {
    k = k + 1;
    n = i + ((k % 2) * 2 - 1) * (k / 2);
    a = n < len ? net[n] : 0;
    if (eql(a, air)) {
      indxs[j++] = n;
    }
  } while (k < 32 && j < 4);
  return j == 4;
}

__host__ __device__
void link(u64* net, u64 len, u64 a_indx, u64 a_slot, u64 b_indx, u64 b_slot) {
  net[a_indx] = set_port(net[a_indx], a_slot, b_indx - a_indx, b_slot);
  net[b_indx] = set_port(net[b_indx], b_slot, a_indx - b_indx, a_slot);
}

__host__ __device__
void unlink(u64 *net, u64 len, u64 a_indx, u64 a_slot) {
  u64 a_node = net[a_indx];
  u64 b_indx = get_dist(a_node, a_slot) + a_indx;
  u64 b_slot = get_slot(a_node, a_slot);
  u64 b_node = net[b_indx];
  if (get_dist(b_node, b_slot) + b_indx == a_indx && get_slot(b_node, b_slot) == a_slot) {
    net[a_indx] = set_port(a_node, a_slot, 0, a_slot);
    net[b_indx] = set_port(b_node, b_slot, 0, b_slot);
  }
}

__host__ __device__
u64 redex_type(u64* net, u64 len, u64 a_indx) {
  u64 a_node = net[a_indx];
  u64 b_indx = get_dist(a_node, 0) + a_indx;
  u64 b_node = net[b_indx];
  if (get_slot(a_node, 0) == 0 && (get_dist(a_node, 0) + get_dist(b_node, 0)) == 0 && !eql(a_node, air)) {
    return get_kind(a_node) == get_kind(b_node) ? 1 : 2;
  } else {
    return 0;
  }
};

__host__ __device__
bool rewrite(u64* net, u64 len, u64 a_indx) {
  u64 a_node = net[a_indx];
  u64 b_indx = a_indx + get_dist(a_node, 0);
  u64 b_node = net[b_indx];
  if (redex_type(net, len, a_indx) == 0) return false;
  if (get_kind(a_node) == get_kind(b_node)) {
    u64 a1_indx = get_dist(net[a_indx], 1) + a_indx;
    u64 a1_slot = get_slot(net[a_indx], 1);
    u64 b1_indx = get_dist(net[b_indx], 1) + b_indx;
    u64 b1_slot = get_slot(net[b_indx], 1);
    link(net, len, a1_indx, a1_slot, b1_indx, b1_slot);
    u64 a2_indx = get_dist(net[a_indx], 2) + a_indx;
    u64 a2_slot = get_slot(net[a_indx], 2);
    u64 b2_indx = get_dist(net[b_indx], 2) + b_indx;
    u64 b2_slot = get_slot(net[b_indx], 2);
    link(net, len, a2_indx, a2_slot, b2_indx, b2_slot);
  } else {
    u64 indxs[4] = {0, 0, 0, 0};
    if (!alloc4(net, len, (a_indx + b_indx) / 2, indxs)) return false;
    u64 c_indx = indxs[0];
    u64 d_indx = indxs[1];
    u64 e_indx = indxs[2];
    u64 f_indx = indxs[3];
    net[c_indx] = new_node(get_kind(b_node), 0,0, f_indx - c_indx, 1, e_indx - c_indx, 1); 
    net[d_indx] = new_node(get_kind(b_node), 0,0, f_indx - d_indx, 2, e_indx - d_indx, 2); 
    net[e_indx] = new_node(get_kind(a_node), 0,0, c_indx - e_indx, 2, d_indx - e_indx, 2);
    net[f_indx] = new_node(get_kind(a_node), 0,0, c_indx - f_indx, 1, d_indx - f_indx, 1);
    link(net, len, c_indx, 0, get_dist(net[a_indx],1) + a_indx, get_slot(net[a_indx],1));
    link(net, len, d_indx, 0, get_dist(net[a_indx],2) + a_indx, get_slot(net[a_indx],2));
    link(net, len, e_indx, 0, get_dist(net[b_indx],2) + b_indx, get_slot(net[b_indx],2));
    link(net, len, f_indx, 0, get_dist(net[b_indx],1) + b_indx, get_slot(net[b_indx],1));
  }
  for (int slot = 0; slot < 3; slot++) {
    unlink(net, len, a_indx, slot);
    unlink(net, len, b_indx, slot);
  }
  net[a_indx] = air;
  net[b_indx] = air;
  return true;
}

__host__ __device__
void move(u64 *net, u64 len, u64 a_indx, u64 b_indx) {
  u64 a_node = net[a_indx];
  u64 b_node = net[b_indx];
  net[b_indx] = inc_ports(a_node, -(b_indx - a_indx));
  net[a_indx] = inc_ports(b_node, -(a_indx - b_indx));
  for (u64 slot = 0; slot < 3; ++slot) {
    u64 a_dist = get_dist(a_node, slot);
    u64 a_slot = get_slot(a_node, slot);
    u64 b_dist = get_dist(b_node, slot);
    u64 b_slot = get_slot(b_node, slot);
    u64 c_indx = a_dist == 0 ? b_indx : a_dist == b_indx - a_indx ? a_indx : a_indx + a_dist;
    u64 d_indx = b_dist == 0 ? a_indx : b_dist == a_indx - b_indx ? b_indx : b_indx + b_dist;
    net[c_indx] = inc_port(net[c_indx], a_slot, b_indx - a_indx);
    net[d_indx] = inc_port(net[d_indx], b_slot, a_indx - b_indx);
  }
}

__host__ __device__
void chill(u64 *net, u64 len) {
  for (u64 i = 0; i < len - 1; i += 2) {
    if (get_force(net[i]) > get_force(net[i + 1])) {
      move(net, len, i, i + 1);
    }
  }
  for (u64 i = 1; i < len - 1; i += 2) {
    if (get_force(net[i]) > get_force(net[i + 1])) {
      move(net, len, i, i + 1);
    }
  }
}

__host__ __device__
bool is_valid(u64 *net, u64 len) {
  for (u64 a_indx = 0; a_indx < len; ++a_indx) {
    for (u64 a_slot = 0; a_slot < 3; ++a_slot) {
      u64 a_node = net[a_indx];
      u64 b_indx = get_dist(a_node, a_slot) + a_indx;
      u64 b_slot = get_slot(a_node, a_slot);
      u64 b_node = net[b_indx];
      if (get_dist(b_node,b_slot) != a_indx - b_indx || get_slot(b_node,b_slot) != a_slot) {
        return false;
      }
    }
  }
  return true;
}

std::vector<u64> redexes(u64 *net, u64 len) {
  std::vector<u64> redexes;
  for (u64 a_indx = 0; a_indx < len; ++a_indx) {
    u64 b_indx = get_dist(net[a_indx], 0) + a_indx;
    if (a_indx <= b_indx && redex_type(net, len, a_indx) > 0) {
      redexes.push_back(a_indx);
    }
  }
  return redexes;
}

u64 reduce_pass(u64 *net, u64 len) {
  std::vector<u64> rdx = redexes(net, len);
  u64 rwt = 0;
  for (u64 i = 0; i < rdx.size(); ++i) {
    if (rewrite(net, len, rdx[i]))  {
      rwt += 1;
    }
  }
  return rwt;
}

// ::::::::::
// :: Misc ::
// ::::::::::

std::string show_slot(u64 node, u64 slot) {
  std::string str;
  str.append(std::to_string(get_dist(node, slot)));
  switch (get_slot(node, slot)) {
    case 0: str.append("a"); break;
    case 1: str.append("b"); break;
    case 2: str.append("c"); break;
  }
  return str;
}

std::string show_node(u64 node) {
  std::string str;
  str.append(std::to_string(get_kind(node)));
  for (int slot = 0; slot < 3; ++slot) {
    str.append(slot > 0 ? " " : "[");
    str.append(show_slot(node, slot));
  }
  str.append("] {");
  str.append(std::to_string(get_force(node)));
  str.append("}");
  return str;
}

std::string plot_nums(std::vector<f64> &nums, std::vector<u64> &cols) {
  std::string str;
  for (uint i = 0; i < nums.size(); ++i) {
    str.append(cols[i] == 0 ? "\x1b[33m" : cols[i] == 1 ? "\x1b[32m" : "\x1b[31m");
    switch ((u64)(floor(fmax(fmin(nums[i],(f64)1),(f64)0) * 8))) {
      case 0: str.append(","); break;
      case 1: str.append("▁"); break;
      case 2: str.append("▂"); break;
      case 3: str.append("▃"); break;
      case 4: str.append("▄"); break;
      case 5: str.append("▅"); break;
      case 6: str.append("▆"); break;
      case 7: str.append("▇"); break;
      case 8: str.append("█"); break;
    }
    str.append("\x1b[0m");
    if (i % 128 == 127 && i < nums.size() - 1) {
      str.append("\n");
    }
  }
  return str;
};

void print_net(u64 *net, u64 len, bool show_nodes, bool show_stats, bool show_heatmap) {
  for (u64 i = 0; i < len; ++i) {
    if (show_nodes && !eql(net[i], air)) {
      std::cout << i << " - " << show_node(net[i]) << std::endl;
    }
  }
  if (show_stats) {
    std::cout << "Valid: " << is_valid(net, len) << std::endl;
  }
  if (show_heatmap) {
    std::vector<f64> nums;
    std::vector<u64> cols;
    for (u64 i = 0; i < len; ++i) {
      nums.push_back(eql(net[i], air) ? 0 : 1.0 / 8.0 + sqrt(abs(get_force(net[i]))) / 64.0);
      cols.push_back(redex_type(net, len, i));
    }
    std::cout << plot_nums(nums, cols) << std::endl;
  }
}

void print_nums(u64 *vec, u64 len) {
  for (u64 i = 0; i < len; ++i) {
    std::cout << vec[i] << " ";
  }
  std::cout << std::endl;
}

__global__
void expand(u64 *src, u64 *dst) {
  int i = blockIdx.x * blockDim.x + threadIdx.x;
  u64 node = src[i];
  dst[i * 2 + 0] = new_node(get_kind(node),
    get_dist(node, 0) * 2, get_slot(node, 0),
    get_dist(node, 1) * 2, get_slot(node, 1),
    get_dist(node, 2) * 2, get_slot(node, 2));
  dst[i * 2 + 1] = air;
}

__global__
void shrink(u64 *src, u64 *dst, u64 *mov) {
  u64 src_indx = blockIdx.x * blockDim.x + threadIdx.x;
  u64 dst_indx = mov[src_indx];
  u64 node = src[src_indx]; 
  if (!eql(node, air)) {
    u64 x_dst_indx = mov[get_dist(node, 0) + src_indx];
    u64 y_dst_indx = mov[get_dist(node, 1) + src_indx];
    u64 z_dst_indx = mov[get_dist(node, 2) + src_indx];
    dst[dst_indx] = new_node(get_kind(node),
      (i64)x_dst_indx - (i64)dst_indx, get_slot(node, 0),
      (i64)y_dst_indx - (i64)dst_indx, get_slot(node, 1),
      (i64)z_dst_indx - (i64)dst_indx, get_slot(node, 2));
  }
}

// ::::::::::
// :: Main ::
// ::::::::::

struct is_node : public thrust::unary_function<u64,u64> {
  __host__ __device__ u64 operator()(u64 node) { return eql(node, air) ? 0 : 1; }
};

const std::vector<u64> ex = {0x0028000a00f08000,0x0028001200b8803b,0x0028001a006c7fff,0x0008001a00207fff,0x0007fff200108001,0x0027fff200088001,0x0027fff600048003,0x0017ffe5fffd8001,0x0017ffc9fffc8002,0x0027ffd600068001,0x0027ffe5fffe7fff,0x0008001a00217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0008001200217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0017ff8600008001,0x0027fffa00018000,0x0008001a00217fe5,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0008001200217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0017ff8600008001,0x0027fffa00018000,0x0017fd26000e8001,0x00280012002a7fff,0x00280012001c7fff,0x00080015fff47fff,0x0007fff200108001,0x0027fff6000c8001,0x0027fff600068001,0x00280015fffe7fff,0x0017ffc5fff47fff,0x0017ff9600008001,0x0027fffa00018000,0x0017ff6a00048001,0x0027fff600017fff,0x0027fc5200057fc4,0x0017fffa00048001,0x0027fff600017fff};  

int main(void) {

  // Creates net on host
  thrust::host_vector<u64> h_net(256);
  thrust::host_vector<u64> h_indx(256);
  thrust::fill(h_net.begin(), h_net.begin() + h_net.size(), air);
  thrust::fill(h_indx.begin(), h_indx.begin() + h_indx.size(), 0);
  for (int i = 0; i < ex.size(); ++i) h_net[i] = ex[i];

  // Sends to GPU
  thrust::device_vector<u64> d_net0 = h_net;
  thrust::device_vector<u64> d_net1(d_net0.size());
  thrust::device_vector<u64> d_indx(d_net0.size());
  
  // Expands
  thrust::fill(d_net1.begin(), d_net1.end(), air);
  expand<<<8,16>>>(thrust::raw_pointer_cast(&d_net0[0]), thrust::raw_pointer_cast(&d_net1[0]));

  // Sends to CPU & prints
  h_net = d_net1;
  print_net(&h_net[0], h_net.size(), true, true, true);

  // Shrinks
  thrust::fill(d_indx.begin(), d_indx.end(), 0);
  thrust::transform_exclusive_scan(d_net1.begin(), d_net1.end(), d_indx.begin(), is_node(), 0, thrust::plus<u64>());
  thrust::fill(d_net0.begin(), d_net0.end(), air);
  shrink<<<8,16>>>(thrust::raw_pointer_cast(&d_net1[0]), thrust::raw_pointer_cast(&d_net0[0]), thrust::raw_pointer_cast(&d_indx[0]));
  
  // Sends to CPU & prints
  h_net = d_net0;
  print_net(&h_net[0], h_net.size(), true, true, true);

  return 0;
}
