#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include <string>
#include <iostream>
#include <vector>

typedef  int64_t i64;
typedef uint64_t u64;
typedef double   f64;

// ::::::::::
// :: Node ::
// ::::::::::

u64 new_node(u64 kind, i64 a_dist, u64 a_slot, i64 b_dist, u64 b_slot, i64 c_dist, u64 c_slot) {
  return (kind << 54)
      | (a_slot << 52) | ((u64)(a_dist + 32768) << 36)
      | (b_slot << 34) | ((u64)(b_dist + 32768) << 18)
      | (c_slot << 16) | ((u64)(c_dist + 32768) <<  0);
}

u64 get_kind(u64 node) {
  return (node >> 54) & 0xFF;
}

i64 get_dist(u64 node, u64 slot) {
  return (i64)((node >> (36 - slot * 18)) & 0xFFFF) - 32768;
}

u64 get_slot(u64 node, u64 slot) {
  return ((node >> (52 - slot * 18))  & 0x3);
}

u64 inc_port(u64 node, u64 slot, i64 delta) {
  return (u64)((i64)node + (delta << (36 - slot * 18)));
}

u64 mov_node(u64 node, i64 delta) {
  return (u64)((i64)node + (-delta << 36) + (-delta << 18) + -delta);
}

u64 set_port(u64 node, u64 slot, i64 new_dist, u64 new_slot) {
  return node & ~((u64)0x3FFFF << (36 - slot * 18)) | (((new_slot << 16) | (u64)(new_dist + 32768)) << (36 - slot * 18));
}

u64 eql(u64 a, u64 b) {
  return a == b;
}

f64 get_force(u64 node) {
  i64 x = get_dist(node, 0);
  i64 y = get_dist(node, 1);
  i64 z = get_dist(node, 2);
  return (f64)((x < 0 ? -1 : 1) * x * x + (y < 0 ? -1 : 1) * y * y + (z < 0 ? -1 : 1) * z * z);
}

const u64 air = new_node(0, 0,0, 0,1, 0,2);

// :::::::::
// :: Net ::
// :::::::::

typedef std::vector<u64> Net;

u64 alloc(Net &net) {
  net.push_back(air);
  return net.size() - 1;
}

void link(Net &net, u64 a_indx, u64 a_slot, u64 b_indx, u64 b_slot) {
  net[a_indx] = set_port(net[a_indx], a_slot, b_indx - a_indx, b_slot);
  net[b_indx] = set_port(net[b_indx], b_slot, a_indx - b_indx, a_slot);
}

void unlink(Net &net, u64 a_indx, u64 a_slot) {
  u64 a_node = net[a_indx];
  u64 b_indx = get_dist(a_node, a_slot) + a_indx;
  u64 b_slot = get_slot(a_node, a_slot);
  u64 b_node = net[b_indx];
  if (get_dist(b_node, b_slot) + b_indx == a_indx && get_slot(b_node, b_slot) == a_slot) {
    net[a_indx] = set_port(a_node, a_slot, 0, a_slot);
    net[b_indx] = set_port(b_node, b_slot, 0, b_slot);
  }
}

bool is_redex(const Net &net, u64 a_indx) {
  u64 a_node = net[a_indx];
  u64 b_indx = get_dist(a_node, 0) + a_indx;
  u64 b_node = net[b_indx];
  return get_slot(a_node, 0) == 0 && (get_dist(a_node, 0) + get_dist(b_node, 0)) == 0 && !eql(a_node, air);
};

void rewrite(Net &net, u64 a_indx) {
  u64 a_node = net[a_indx];
  u64 b_indx = a_indx + get_dist(a_node, 0);
  u64 b_node = net[b_indx];
  if (get_kind(a_node) == get_kind(b_node)) {
    u64 a1_indx = get_dist(net[a_indx], 1) + a_indx;
    u64 a1_slot = get_slot(net[a_indx], 1);
    u64 b1_indx = get_dist(net[b_indx], 1) + b_indx;
    u64 b1_slot = get_slot(net[b_indx], 1);
    link(net, a1_indx, a1_slot, b1_indx, b1_slot);
    u64 a2_indx = get_dist(net[a_indx], 2) + a_indx;
    u64 a2_slot = get_slot(net[a_indx], 2);
    u64 b2_indx = get_dist(net[b_indx], 2) + b_indx;
    u64 b2_slot = get_slot(net[b_indx], 2);
    link(net, a2_indx, a2_slot, b2_indx, b2_slot);
  } else {
    u64 c_indx = alloc(net);
    u64 d_indx = alloc(net);
    u64 e_indx = alloc(net);
    u64 f_indx = alloc(net);
    net[c_indx] = new_node(get_kind(b_node), 0,0, f_indx - c_indx, 1, e_indx - c_indx, 1); 
    net[d_indx] = new_node(get_kind(b_node), 0,0, f_indx - d_indx, 2, e_indx - d_indx, 2); 
    net[e_indx] = new_node(get_kind(a_node), 0,0, c_indx - e_indx, 2, d_indx - e_indx, 2);
    net[f_indx] = new_node(get_kind(a_node), 0,0, c_indx - f_indx, 1, d_indx - f_indx, 1);
    link(net, c_indx, 0, get_dist(net[a_indx],1) + a_indx, get_slot(net[a_indx],1));
    link(net, d_indx, 0, get_dist(net[a_indx],2) + a_indx, get_slot(net[a_indx],2));
    link(net, e_indx, 0, get_dist(net[b_indx],2) + b_indx, get_slot(net[b_indx],2));
    link(net, f_indx, 0, get_dist(net[b_indx],1) + b_indx, get_slot(net[b_indx],1));
  }
  for (int slot = 0; slot < 3; slot++) {
    unlink(net, a_indx, slot);
    unlink(net, b_indx, slot);
  }
  net[a_indx] = air;
  net[b_indx] = air;
}

void move(Net &net, u64 a_indx, u64 b_indx) {
  u64 a_node = net[a_indx];
  u64 b_node = net[b_indx];
  net[b_indx] = mov_node(a_node, b_indx - a_indx);
  net[a_indx] = mov_node(b_node, a_indx - b_indx);
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

bool is_valid(const Net &net) {
  for (u64 a_indx = 0; a_indx < net.size(); ++a_indx) {
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

/*
// TODO: port from JS
void reduce_pass(Net &net) => {
  u32 rdx = redexes(net);
  var rwt = 0;
  for (var i = 0; i < rdx.length; ++i) {
    if (!only_ani || kin(net[rdx[i][0]]) === kin(net[rdx[i][1]])) {
      if (rewrite(net, rdx[i][0])) {
        rwt += 1;
      }
    }
  }
  return rwt;
};
*/


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

std::string plot_nums(std::vector<f64> &nums, std::vector<bool> &mark) {
  std::string str;
  for (uint i = 0; i < nums.size(); ++i) {
    str.append(mark[i] ? "\x1b[32m" : "\x1b[31m");
    switch ((u64)(floor(max(min(nums[i],(f64)1),(f64)0) * 8))) {
      case 0: str.append(" "); break;
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
  }
  return str;
};

void print_net(const Net &net) {
  std::vector<f64> nums;
  std::vector<bool> mark;
  for (u64 i = 0; i < net.size(); ++i) {
    /*std::cout << i << " - " << show_node(net[i]) << std::endl;*/
    nums.push_back(1.0 / 8.0 + sqrt(abs(get_force(net[i]))) / 64.0);
    mark.push_back(is_redex(net, i));
  }
  std::cout << plot_nums(nums, mark) << std::endl;
}

// ::::::::::
// :: Main ::
// ::::::::::

const std::vector<u64> net = {0x0028000a00f08000,0x0028001200b8803b,0x0028001a006c7fff,0x0008001a00207fff,0x0007fff200108001,0x0027fff200088001,0x0027fff600048003,0x0017ffe5fffd8001,0x0017ffc9fffc8002,0x0027ffd600068001,0x0027ffe5fffe7fff,0x0008001a00217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0008001200217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0017ff8600008001,0x0027fffa00018000,0x0008001a00217fe5,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0008001200217ff8,0x0007fff200088001,0x0027fff200108002,0x0017ffe6000c8004,0x0027ffe6000e8001,0x0028001a000a7fff,0x0057ffc5fff47fff,0x0027ffc5fff57ffe,0x0017ff8600008001,0x0027fffa00018000,0x0017fd26000e8001,0x00280012002a7fff,0x00280012001c7fff,0x00080015fff47fff,0x0007fff200108001,0x0027fff6000c8001,0x0027fff600068001,0x00280015fffe7fff,0x0017ffc5fff47fff,0x0017ff9600008001,0x0027fffa00018000,0x0017ff6a00048001,0x0027fff600017fff,0x0027fc5200057fc4,0x0017fffa00048001,0x0027fff600017fff};  

int main(void) {
  /*std::cout << show_node(inc_port(set_port(air, 2, 7,0), 0, 3)) << std::endl;*/

  print_net(net);
  
  return 0;
}
