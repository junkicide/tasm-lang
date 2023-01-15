use std::collections::{HashMap, HashSet};

use crate::cfg::{self, ControlFlowGraph, Variable};

pub fn convert(cfg: &mut ControlFlowGraph) {
    add_annotations(cfg);
    rename_variables(cfg);
}

/// Modifies a ControlFlowGraph such that:
///  - basic blocks get an explicit parameter list `params` which
///    corresponds to the free variables used in that basic block.
///  - edges get an annotation listing the free variables that their
///    originating basic blocks must supply.
fn add_annotations(cfg: &mut ControlFlowGraph) {
    let mut visited: HashSet<usize> = HashSet::new();
    let mut active_set: Vec<usize> = vec![cfg.entrypoint];

    while !active_set.is_empty() {
        let mut successors = vec![];

        // visit members of active set
        for member_index in active_set {
            let mut member = &mut cfg.nodes[member_index];

            // find free variables
            let mut free_variables: Vec<Variable> = vec![];
            let mut defined_variables = vec![];
            for statement in member.stmts.iter() {
                match &statement.expr {
                    cfg::Expr::Var(v) => {
                        if !free_variables.contains(&v) && !defined_variables.contains(&v) {
                            free_variables.push(v.to_owned());
                        }
                    }
                    cfg::Expr::Lit(_) => {}
                };
                defined_variables.push(&statement.var);
            }

            // find incoming edges for `member`
            // iterate over all of them
            // and add annotation
            for incoming_edge in cfg
                .edges
                .iter_mut()
                .filter(|e| e.destination == member_index)
            {
                incoming_edge.annotations = free_variables.clone();
            }

            // make implicit parameter explicit
            member.params = free_variables.clone();

            // mark visited
            visited.insert(member_index);

            // collect successors
            for successor in cfg
                .edges
                .iter()
                .filter(|e| e.source == member_index)
                .map(|e| e.destination)
            {
                successors.push(successor);
            }
        }

        // prepare for next iteration:
        // prune successor set and assign to active set
        active_set = successors
            .into_iter()
            .filter(|s| !visited.contains(s))
            .collect();
    }
}

fn rename_variables(cfg: &mut ControlFlowGraph) {
    let mut visited: HashSet<usize> = HashSet::new();
    let mut active_set: Vec<usize> = vec![cfg.entrypoint];
    let mut renaming_dictionary = HashMap::<String, String>::new();
    let mut names_already_used: Vec<String> = vec![];

    let mut counter: usize = 0;
    let mut name_gen = |s: &str| {
        let tmp = format!("{}_{}", s, counter);
        counter += 1;
        tmp
    };

    while !active_set.is_empty() {
        let mut successors = vec![];

        // visit members of active set and rename each occurrence of
        // variables
        for member_index in active_set {
            let member = &mut cfg.nodes[member_index];

            // if the basic block has parameters
            for param in member.params.iter_mut() {
                let data_type = param.data_type.to_owned();

                // this name was already used; we need to get a new one
                if names_already_used.contains(&param.name) {
                    let new_name = name_gen(&param.name);
                    renaming_dictionary.insert(param.name.clone(), new_name.clone());
                    // apply name change
                    *param = cfg::Variable {
                        name: new_name.clone(),
                        data_type,
                    };
                    // percolate name change
                    for let_statement in member.stmts.iter_mut() {
                        match &mut let_statement.expr {
                            cfg::Expr::Var(var) => {
                                var.name = new_name.clone();
                            }
                            cfg::Expr::Lit(_) => {}
                        }
                        if let_statement.var.name == *new_name {
                            // expressions following this let-
                            // statement use the new value
                            break;
                        }
                    }
                    // keep track of this new name
                    names_already_used.push(new_name);
                }
                // first occurence
                else {
                    names_already_used.push(param.name.clone());
                }
            }

            // iterate over all let-statements
            for statement_index in 0..member.stmts.len() {
                let let_stmt_copy = member.stmts[statement_index].clone();
                let var_name = let_stmt_copy.var.name.clone();

                // this name was already used, get a new one
                if names_already_used.contains(&var_name) {
                    let new_name = name_gen(&var_name);
                    renaming_dictionary.insert(let_stmt_copy.var.name.clone(), new_name.clone());

                    // apply and percolate name change
                    for let_statement in member.stmts.iter_mut().skip(statement_index) {
                        match &mut let_statement.expr {
                            cfg::Expr::Var(var) => {
                                var.name = new_name.clone();
                            }
                            cfg::Expr::Lit(_) => {}
                        }
                        if let_statement.var.name == *new_name {
                            // expressions following this let-
                            // statement use the new value
                            break;
                        }
                    }

                    // keep track of new name
                    names_already_used.push(new_name.clone());
                }
                // first occurence
                else {
                    names_already_used.push(var_name.clone());
                }
            }

            // mark visited
            visited.insert(member_index);

            // collect successors
            for successor in cfg
                .edges
                .iter()
                .filter(|e| e.source == member_index)
                .map(|e| e.destination)
            {
                successors.push(successor);
            }
        }

        // prepare for next iteration:
        // prune successor set and assign to active set
        active_set = successors
            .into_iter()
            .filter(|successor| !visited.contains(successor))
            .collect();
    }
}

// fn rename_variable(var: &mut cfg::Variable, renaming_dictionary: &HashMap<String, String>) {
//     if let Some(new_name) = renaming_dictionary.get(&var.name) {
//         let data_type = var.data_type.to_owned();
//         *var = cfg::Variable {
//             name: new_name.clone(),
//             data_type,
//         };
//     }
// }

// fn rename_expr(expr: &mut cfg::Expr, renaming_dictionary: &HashMap<String, String>) {
//     match expr {
//         cfg::Expr::Var(var) => rename_variable(var, renaming_dictionary),
//         cfg::Expr::Lit(_) => (),
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{
        ast::{DataType, ExprLit},
        cfg::{BasicBlock, Edge, Expr, LetStmt},
    };

    use super::*;

    fn gen_cfg() -> cfg::ControlFlowGraph {
        // ```
        // block_0:
        //   foo = bar
        //   call block_1
        //
        // block_1:
        //   baz = foo
        //   foo = 3
        // ```
        //
        // should be mapped to:
        //
        // ```
        // block_0(bar):
        //   foo = bar
        //   call block_1(foo)
        //
        // block_1(foo):
        //   baz = foo
        //   foo_0 = 3
        // ```
        let foo_var = Variable {
            name: "foo".to_string(),
            data_type: DataType::U32,
        };

        let bar_var = Variable {
            name: "bar".to_string(),
            data_type: DataType::U32,
        };

        let baz_var = Variable {
            name: "bar".to_string(),
            data_type: DataType::U32,
        };

        let mut cfg = ControlFlowGraph::default();

        // block_0
        cfg.nodes.push(BasicBlock {
            index: 0,
            params: vec![],
            stmts: vec![LetStmt {
                var: foo_var.clone(),
                expr: Expr::Var(bar_var),
            }],
        });

        // block_1
        cfg.nodes.push(BasicBlock {
            index: 1,
            params: vec![foo_var.clone()],
            stmts: vec![
                LetStmt {
                    var: baz_var.clone(),
                    expr: Expr::Var(foo_var.clone()),
                },
                LetStmt {
                    var: foo_var.clone(),
                    expr: Expr::Lit(ExprLit::CU32(3)),
                },
            ],
        });

        // edges
        cfg.edges.push(Edge {
            source: 0,
            destination: 1,
            annotations: vec![],
        });

        cfg.exitpoint = 1;

        cfg
    }

    #[test]
    fn simple_ssa_test() {
        let mut cfg = gen_cfg();
        add_annotations(&mut cfg);

        // todo: assert that annotations happened

        // println!("{:#?}", acfg);

        rename_variables(&mut cfg);

        println!("{:#?}", cfg);
    }
}
