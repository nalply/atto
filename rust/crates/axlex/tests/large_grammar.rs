#![feature(lazy_cell, const_type_name, const_option)]
#![recursion_limit = "300"]

use axlex::lexer;
use std::mem::size_of_val;

#[test]
pub fn test() {
  lexer! {
    large<()> {
      ALL: [
        ws(r"\s+"), comment(r"//.*?$|/\*[\s\S]*?\*/")
      ],
      init: [
        let("let"), mut("mut"), use("use"), mod("mod"), struct("struct"),
        enum("enum"), match("match"), if("if"), else("else"), but("but"),
        while("while"), for("for"), loop("loop"), break("break"),
        continue("continue"), return("return"), fn("fn"), impl("impl"),
        trait("trait"), pub("pub"), crate("crate"), self("self"),
        static("static"), async("async"), await("await"), move("move"),
        unsafe("unsafe"), dyn("dyn"), where("where"), as("as"),
        in("in"), ref("ref"), type("type"), sizeof("sizeof"), die("die"),
        alignof("alignof"), typeof("typeof"), throw("throw"), def("def"),
        lambda("lambda"), yield("yield"), effect("effect"), from("from"),
        default("default"), do("do"), catch("catch"), until("until"),
        finally("finally"), package("package"), push("push"), croak("croak"),
        export("export"), throws("throws"), extends("extends"), ask("ask"),
        var("var"), val("val"), forget("forget"), cleave("cleave"),
        abstract("abstract"), const("const"), meld("meld"),
        public("public"), private("private"), protected("protected"),
        transient("transient"), nudge("nudge"), please("please"), _priv("priv"),
        volatile("volatile"), native("native"), meow("meow"), grok("grok"),
        class("class"), interface("interface"), koan("koan"), merge("merge"),
        implements("implements"), this("this"), that("that"), it("it"),
        super("super"), goto("goto"), jump("jump"), strict("strict"),
        new("new"), instanceof("instanceof"), synchronized("synchronized"),
        strictfp("strictfp"), import("import"), insert("insert"), who("who"),
        assert("assert"), module("module"), requires("requires"), why("why"),
        exports("exports"), opens("opens"), to("to"), by("by"), fun("fun"),
        select("select"), join("join"), delete("delete"), update("update"),
        exists("exists"), quine("quine"), spoon("spon"), fork("fork"),
        call("call"), thread("thread"), fi("fi"), esac("esac"), say("say"),
        sub("sub"), my("my"), has("has"), our("our"), given("given"),
        plus(r"\+"), minus(r"-"), asterisk(r"\*"), slash(r"/"), x("x"),
        percent(r"%"), ampersand(r"&"), pipe(r"\|"), caret(r"\^"),
        equals("=="), exclamation("!"), turbo_fish("::<>"), tilde("~"),
        smart_match("~~"), range(r"\.\."), exclusive_range(r"\.\.\."),
        spaceship("<=>"), hyper_start("«"), hyper_end("»"), cross("×"),
        inner_product("·"), roll("⌽"), runic_merge("᚛"), runic_cross("ᚄ"),
        integer(r"\d+"), floating_point(r"\d+\.\d+"),
        char_lit(r"'[^']*x'"), string_lit(r#""[^"]*""#),
      ]
    };
  }

  eprintln!("rule count: {}", RULES.len());
  eprintln!("rule list size: {}", size_of_val(&RULES));
  let size_of_rule_names =
    RULES.iter().fold(0, |acc, rule| acc + size_of_val(rule.rule_name));
  eprintln!("size of rule names: {size_of_rule_names}");
}
