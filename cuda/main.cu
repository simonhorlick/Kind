#include <stdio.h>
#include <stdint.h>
#include <string>
#include <iostream>
#include <vector>

typedef  int64_t i64;
typedef uint64_t u64;

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

const u64 air = new_node(0, 0,0, 0,1, 0,2);

u64 alloc(std::vector<u64> &net) {
  net.push_back(air);
  return net.size() - 1;
}

void link(std::vector<u64> &net, u64 a_indx, u64 a_slot, u64 b_indx, u64 b_slot) {
  net[a_indx] = set_port(net[a_indx], a_slot, b_indx - a_indx, b_slot);
  net[b_indx] = set_port(net[b_indx], b_slot, a_indx - b_indx, a_slot);
}

void unlink(std::vector<u64> &net, u64 a_indx, u64 a_slot) {
  u64 a_node = net[a_indx];
  u64 b_indx = get_dist(a_node, a_slot) + a_indx;
  u64 b_slot = get_slot(a_node, a_slot);
  u64 b_node = net[b_indx];
  if (get_dist(b_node, b_slot) + b_indx == a_indx && get_slot(b_node, b_slot) == a_slot) {
    net[a_indx] = set_port(a_node, a_slot, 0, a_slot);
    net[b_indx] = set_port(b_node, b_slot, 0, b_slot);
  }
}

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
  str.append("]");
  return str;
}


/*var nod = (k,ad,as,bd,bs,cd,cs) => (1<<31)|(k<<30)|(as<<28)|((ad+128)<<20)|(bs<<18)|((bd+128)<<10)|(cs<<8)|(cd+128);*/
/*var kin = (nod) => (nod >>> 30) & 0x1; // get kind*/
/*var dst = (nod,s) => ((nod >>> (20 - s * 10)) & 0xFF) - 128; // get target distance of port `s`*/
/*var slt = (nod,s) => ((nod >>> (28 - s * 10)) & 0x3); // get target slot of port `s`*/
/*var add = (nod,s,k) => nod + (k << (20 - s * 10)); // add k to slot s*/
/*var mov = (nod,d) => nod + (-d << 20) + (-d << 10) + -d; // dd d to all slots*/
/*var set = (nod,s,D,S) => nod&~(0x3FF<<(20-s*10))|(((S<<8)|(D+128))<<(20-s*10)); // set ptr on slot s to ti,ts*/
/*var eql = (a,b) => a === b;*/
/*var air = nod(0, 0,0, 0,1, 0,2);*/
/*var ela = (n) => Math.sign(n) * n * n;*/
/*var pow = (nod) => ela(dst(nod,0)) + ela(dst(nod,1)) + ela(dst(nod,2));*/
/*var max = (nod) => Math.max(Math.abs(dst(nod,0)), Math.max(Math.abs(dst(nod,1)), Math.abs(dst(nod,2))));*/
/*var sho = (nod) => eql(nod, air) ? "~" : kin(nod) + "[" + dst(nod,0) + "abc"[slt(nod,0)] + "|" + dst(nod,1) + "abc"[slt(nod,1)] + "|" + dst(nod,2) + "abc"[slt(nod,2)] + "] " + "{" + pow(nod).toFixed(2) + "}";*/
/*var str = net => net.map((k,i) => "| " + ("0000" + i).slice(-4) + " : " + sho(k)).join("\n");*/

int main(void) {
  std::cout << show_node(inc_port(set_port(air, 2, 7,0), 0, 3)) << std::endl;
  /*printf("hi");*/
  
  return 0;
}
