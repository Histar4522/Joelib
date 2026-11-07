use crate::ast::*;

pub struct Explainer;

impl Explainer {
    pub fn explain_program(stmts: &[Stmt]) -> String {
        let mut narrative = Vec::new();
        narrative.push("--- The Chronicles of Joe: An Analytical Exposition ---\n".to_string());

        for (i, stmt) in stmts.iter().enumerate() {
            let step = i + 1;
            narrative.push(format!("{}. {}", step, Self::explain_stmt(stmt, 0)));
        }

        narrative.push("\n--- End of Joe's Chapter ---".to_string());
        narrative.join("\n")
    }

    pub fn explain_stmt(stmt: &Stmt, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match &stmt.kind {
            StmtKind::Remember { name, init } => {
                format!("{}Joe commits '{}' to memory, evaluated from '{}'.", pad, name, Self::explain_expr(init))
            }
            StmtKind::Assign { target, value } => {
                format!("{}Joe alters his perception of '{}' to become '{}'.", pad, Self::explain_expr(target), Self::explain_expr(value))
            }
            StmtKind::Yell { expr, to } => {
                if let Some(t) = to {
                    format!("{}Joe projects his voice into '{}': '{}'.", pad, Self::explain_expr(t), Self::explain_expr(expr))
                } else {
                    format!("{}Joe projects his voice into the uncaring void: '{}'.", pad, Self::explain_expr(expr))
                }
            }
            StmtKind::Whisper { expr, to } => {
                if let Some(t) = to {
                    format!("{}Joe quietly murmurs to '{}': '{}'.", pad, Self::explain_expr(t), Self::explain_expr(expr))
                } else {
                    format!("{}Joe quietly murmurs to himself: '{}'.", pad, Self::explain_expr(expr))
                }
            }
            StmtKind::Listen { name, from } => {
                if let Some(f) = from {
                    format!("{}Joe listens intently to '{}', committing the words to '{}'.", pad, Self::explain_expr(f), name)
                } else {
                    format!("{}Joe pauses to listen to external reality, storing whatever he hears into '{}'.", pad, name)
                }
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                let mut s = format!("{}Joe ponders condition '{}':\n", pad, Self::explain_expr(cond));
                s.push_str(&format!("{}  If true, Joe undertakes:\n", pad));
                for sub in then_branch {
                    s.push_str(&format!("{}    * {}\n", pad, Self::explain_stmt(sub, 0)));
                }
                if let Some(elses) = else_branch {
                    s.push_str(&format!("{}  Otherwise, Joe settles for:\n", pad));
                    for sub in elses {
                        s.push_str(&format!("{}    * {}\n", pad, Self::explain_stmt(sub, 0)));
                    }
                }
                s.push_str(&format!("{}  And so, that matter is settled.", pad));
                s
            }
            StmtKind::While { cond, body } => {
                let mut s = format!("{}Joe enters an obsessive cycle so long as '{}' persists:\n", pad, Self::explain_expr(cond));
                for sub in body {
                    s.push_str(&format!("{}  * {}\n", pad, Self::explain_stmt(sub, 0)));
                }
                s.push_str(&format!("{}Joe eventually moves on.", pad));
                s
            }
            StmtKind::For { item, iterable, body } => {
                let mut s = format!("{}Joe rummages through '{}', considering each element as '{}':\n", pad, Self::explain_expr(iterable), item);
                for sub in body {
                    s.push_str(&format!("{}  * {}\n", pad, Self::explain_stmt(sub, 0)));
                }
                s.push_str(&format!("{}Joe exhausts the collection and moves on.", pad));
                s
            }
            StmtKind::GiveUp => {
                format!("{}Joe abandons all effort and gives up on the current ordeal (loop break).", pad)
            }
            StmtKind::TryAnyway => {
                format!("{}Joe stubbornly tries anyway, jumping straight to the next iteration (loop continue).", pad)
            }
            StmtKind::Deliver(val) => {
                match val {
                    Some(e) => format!("{}Joe delivers his conclusion: '{}'.", pad, Self::explain_expr(e)),
                    None => format!("{}Joe concludes his routine with nothing to show for it.", pad),
                }
            }
            StmtKind::Function { name, params, body } => {
                let mut s = format!("{}Joe establishes a new habit named '{}' taking parameters ({}) with inner logic:\n", pad, name, params.join(", "));
                for sub in body {
                    s.push_str(&format!("{}  * {}\n", pad, Self::explain_stmt(sub, 0)));
                }
                s.push_str(&format!("{}That is how Joe executes this habit.", pad));
                s
            }
            StmtKind::Class { name, fields, methods } => {
                let mut s = format!("{}Joe conceptualizes a social archetype '{}' possessing traits [{}]:\n", pad, name, fields.join(", "));
                for m in methods {
                    s.push_str(&format!("{}  - {}\n", pad, Self::explain_stmt(m, 0)));
                }
                s.push_str(&format!("{}Such is the nature of a '{}'.", pad, name));
                s
            }
            StmtKind::Module { name, body } => {
                let mut s = format!("{}Joe establishes an autonomous domain named '{}':\n", pad, name);
                for sub in body {
                    s.push_str(&format!("{}  * {}\n", pad, Self::explain_stmt(sub, 0)));
                }
                s.push_str(&format!("{}That encompasses the domain of '{}'.", pad, name));
                s
            }
            StmtKind::Panic(e) => {
                format!("{}Joe suffers an existential breakdown over '{}'.", pad, Self::explain_expr(e))
            }
            StmtKind::CallFor(module) => {
                format!("{}Joe summons external package '{}' to aid his mental burdens.", pad, module)
            }
            StmtKind::Expr(e) => {
                format!("{}Joe contemplates expression '{}'.", pad, Self::explain_expr(e))
            }
        }
    }

    pub fn explain_expr(expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Nothing => "the void (nothing)".to_string(),
            ExprKind::Bool(true) => "Joe agrees (true)".to_string(),
            ExprKind::Bool(false) => "Joe disagrees (false)".to_string(),
            ExprKind::Number(n) => format!("{}", n),
            ExprKind::Text(s) => format!("\"{}\"", s),
            ExprKind::Ident(id) => id.clone(),
            ExprKind::JThis => "Joe himself (jthis)".to_string(),
            ExprKind::List(elems) => {
                let list_str: Vec<String> = elems.iter().map(Self::explain_expr).collect();
                format!("[{}]", list_str.join(", "))
            }
            ExprKind::Bag(entries) => {
                let bag_str: Vec<String> = entries.iter().map(|(k, v)| format!("{}: {}", k, Self::explain_expr(v))).collect();
                format!("{{{}}}", bag_str.join(", "))
            }
            ExprKind::Unary { op, expr: inner } => {
                match op {
                    UnaryOp::Neg => format!("negative {}", Self::explain_expr(inner)),
                    UnaryOp::Not => format!("not ({})", Self::explain_expr(inner)),
                }
            }
            ExprKind::Binary { left, op, right } => {
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                };
                format!("({} {} {})", Self::explain_expr(left), op_str, Self::explain_expr(right))
            }
            ExprKind::Call { callee, args } => {
                let arg_strs: Vec<String> = args.iter().map(Self::explain_expr).collect();
                format!("{}({})", Self::explain_expr(callee), arg_strs.join(", "))
            }
            ExprKind::GetField { object, name } => {
                format!("{}.{}", Self::explain_expr(object), name)
            }
            ExprKind::GetIndex { object, index } => {
                format!("{}[{}]", Self::explain_expr(object), Self::explain_expr(index))
            }
            ExprKind::New { class_name, args } => {
                let arg_strs: Vec<String> = args.iter().map(Self::explain_expr).collect();
                format!("a new {}({})", class_name, arg_strs.join(", "))
            }
        }
    }
}
