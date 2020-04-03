var fmc = require("./FormalityCore.js");
var fs = require("fs");
var code = fs.readFileSync("./Formality.fmc", "utf8");

// Parses module
//var module = fmc.parse_mod(code, 0);

//// Normalizes and type-checks all terms
//for (var name in module) {
  //console.log("name:", name);
  //try {
    //console.log("type:", fmc.stringify_trm(fmc.typecheck(module[name].term, module[name].type, module)));
  //} catch (e) {
    //console.log("type:", e);
  //}
  //console.log("");
//};

const nil = nil => ext => nil;
const ext = h => t => nil => ext => ext(h)(t);
const be  = be => b0 => b1 => be;
const b0  = x => be => b0 => b1 => b0(x);
const b1  = x => be => b0 => b1 => b1(x);

function char_to_lambda(chr) {
  var lam = be;
  var num = chr.charCodeAt(0);
  for (var i = 0; i < 32; ++i) {
    if (((num >>> (32 - i - 1)) & 1) === 0) {
      lam = b0(lam);
    } else {
      lam = b1(lam);
    }
  };
  return lam;
};

function string_to_lambda(str, i = 0) {
  if (i === str.length) {
    return nil;
  } else {
    var head = char_to_lambda(str[i]);
    var tail = string_to_lambda(str, i + 1);
    return ext(head)(tail);
  }
};

function lambda_to_char(lam) {
  function go(lam, n = 0, k = 1) {
    var case_be = () => n;
    var case_b0 = (pred) => () => go(pred, n + 0, k * 2);
    var case_b1 = (pred) => () => go(pred, n + k, k * 2);
    return lam(case_be)(case_b0)(case_b1)();
  };
  return String.fromCharCode(go(lam));
};

function lambda_to_string(lam) {
  var case_nil = "";
  var case_ext = (h) => (t) => lambda_to_char(h) + lambda_to_string(t);
  return lam(case_nil)(case_ext);
};

function term_to_js(term, vars = fmc.Nil(), depth = 0) {
  switch (term.ctor) {
    case "Var":
      var got = fmc.find(vars, (x,i) => i === term.indx);
      if (got) {
        return got.value;
      } else {
        return "'{{var" + (depth - term.indx - 1) + "}}'";
      }
    case "Ref":
      if (!term.name) throw "aff";
      return "ref('"+term.name+"')";
    case "Typ":
      return "null";
    case "All":
      return "null";
    case "Lam":
      if (term.eras) {
        return term_to_js(fmc.subst(term.body, fmc.Ref("__ERASED__"), 0), vars, depth);
      } else {
        var name = "x"+depth;
        var body = term_to_js(term.body, fmc.Ext(name, vars), depth + 1);
        return "(("+name+")=>"+body+")";
      };
    case "App":
      if (term.eras) {
        return term_to_js(term.func, vars, depth);
      } else {
        var func = term_to_js(term.func, vars, depth);
        var argm = term_to_js(term.argm, vars, depth);
        return func+"("+argm+")";
      }
    case "Let":
      var name = "x"+depth;
      var expr = term_to_js(term.expr, vars, depth);
      var body = term_to_js(term.body, fmc.Ext(name, vars), depth + 1);
      return "(("+name+")=>"+body+")("+expr+")";
    case "Ann":
      return term_to_js(term.expr, vars, depth);
  }
};

function module_to_js(module) {
  var code = "(function(){\n";
  code += "  var ref = (name) => got[name] || (got[name] = lib[name]());\n";
  code += "  var got = {};\n";
  code += "  var lib = {};\n";
  for (var name in module) {
    code += "  lib." + name + " = () => " + term_to_js(module[name].term) + ";\n";
  };
  code += "  return ref;\n";
  code += "})()";
  return code;
};

var module = fmc.parse_mod(code);
var jscode = module_to_js(module);
var func = eval(jscode)("example_1");
var argm = string_to_lambda("(A : Type) -> (y : A) -> A");

console.log(lambda_to_string(func(argm)));
