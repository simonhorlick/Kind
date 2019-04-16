const to_diffnet = net => { 
  var diffnet = [];
  for (var i = 0; i < net.nodes.length; ++i) {
    var node = net.nodes[i];
    diffnet.push([node.label,
      node.ports[0].addr - i, node.ports[0].port,
      node.ports[1].addr - i, node.ports[1].port,
      node.ports[2].addr - i, node.ports[2].port]);
  }
  return diffnet;
};

module.exports = {to_diffnet};
