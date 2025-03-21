use std::collections::HashMap;
use std::rc::Rc;
use rustc_middle::mir;
use rustc_span::{BytePos, Span};

#[derive(Debug, Clone)]
pub enum BlockStatement<'tcx> {
    Statement(mir::Statement<'tcx>),
    TerminatorKind(mir::TerminatorKind<'tcx>)
}

// Hold states for the reentrancy
pub struct ReentrancyChecker<'tcx> {
    // The block statements are belong to a function
    pub block_statements: HashMap<mir::BasicBlock, Vec<BlockStatement<'tcx>>>,
    // The function call transfers tokens in solana contract
    pub function_lamport_transfer: HashMap<mir::BasicBlock, Rc<str>>,
    // The temporary variable holds the balance of an user in the solana contract
    pub temporary_variable_for_balance: Option<mir::Place<'tcx>>,
    // Check for detecting the variable holding the balance of an user in the solana contract
    pub check_for_balance_variable: bool,
    //  Current assign destination in the statement
    pub current_assign_destination: Option<mir::Place<'tcx>>,
    // The starting spans contain reentrancy codes
    pub starting_reentrancy_span: BytePos,
    // The ending spans contain reentrancy codes
    pub ending_reentrancy_span: BytePos
}

impl<'tcx> ReentrancyChecker<'tcx> {
    pub fn new() -> ReentrancyChecker<'tcx> {
        ReentrancyChecker {
            block_statements: HashMap::default(),
            function_lamport_transfer: HashMap::default(),
            temporary_variable_for_balance: None,
            check_for_balance_variable: false,
            current_assign_destination: None,
            starting_reentrancy_span: BytePos(0),
            ending_reentrancy_span: BytePos(0)
        }
    }

    /// Check if the reentrancy happens. The reentrancy will possibly happens if the following executions
    /// happen. First, a ``LOAD`` instruction occurs. Second, the ``TRANSFER`` instruction occurs.
    /// Lastly, a ``STORE`` instruction executes, interacting with the same location accessed by
    /// the former ``LOAD`` instruction.
    pub fn check(&self) -> bool {
        info!("Check for reentrancy");
        let mut is_reentrancy = false;
        if self.function_lamport_transfer.is_empty() {
            return is_reentrancy;
        }
        if let Some((last_bb, _)) = self.function_lamport_transfer.iter().last() {
            info!("Last function lamport {:?}", last_bb);
            info!("Variable for balance {:?}", self.temporary_variable_for_balance);
            for (bb, block_statements) in &self.block_statements {
                if bb <= last_bb {
                    continue;
                }
                info!("bb {:?}, last_bb {:?}, greater {:?}", bb, last_bb, bb > last_bb);
                for block_statement in block_statements {
                    // If the balance is assigned to a constant
                    if let BlockStatement::Statement(statement) = block_statement {
                        let mir::Statement { kind, .. } = statement;
                        let status = self.visit_reentrancy_statement(kind);
                        is_reentrancy = status || is_reentrancy;
                        if is_reentrancy {
                            break;
                        }
                    }
                    // If the balance is related to arithmetic operations. E.g., balance -= amount
                    if let BlockStatement::TerminatorKind(kind) = block_statement {
                        let status = self.visit_reentrancy_terminator(kind);
                        is_reentrancy = status || is_reentrancy;
                        if is_reentrancy {
                            break;
                        }
                    }
                }
            }
        }
        return is_reentrancy;
    }
    
    fn visit_reentrancy_terminator(&self, kind: &mir::TerminatorKind<'_>) -> bool {
        if let mir::TerminatorKind::Assert { msg, .. } = kind {
            if let mir::AssertKind::Overflow(mir::BinOp::Sub, ref left_operand, _) = **msg {
                if let mir::Operand::Copy(place) = left_operand {
                    if let Some(temporary_place) = self.temporary_variable_for_balance {
                        if temporary_place.local == place.local {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    fn visit_reentrancy_statement(&self, kind: &mir::StatementKind<'_>) -> bool {
        if let mir::StatementKind::Assign(box (place, _)) = kind {
            if let Some(temporary_place) = self.temporary_variable_for_balance {
                if temporary_place.local == place.local {
                    return true;
                }
            }
        }
        return false;
    }
    
}

// Hold states for the bad radomness
pub struct BadrandomnessChecker {
    // Check if the rand lib is used
    pub check_for_rand_lib: bool,
     // The span contains codes related to bad randomness
     pub bad_randomness_span: Span,
}

impl BadrandomnessChecker {
    pub fn new() -> BadrandomnessChecker {
        return BadrandomnessChecker { 
            check_for_rand_lib: false, 
            bad_randomness_span: rustc_span::DUMMY_SP
        }
    }

    /// Check if the bad randomness happens. The bad randomness will possibly happens if 
    /// ``solana_program::sysvar::clock::Clock`` is used
    pub fn check(&self) -> bool {
        return self.check_for_rand_lib;
    }
}

// Hold states for the time manipulation
pub struct TimeManipulationChecker {
    // Check if the clock lib is used
    pub check_for_clock_lib: bool,
     // The span contains codes related to time manipulation
     pub time_manipulation_span: Span,
}

impl TimeManipulationChecker {
    pub fn new() -> TimeManipulationChecker {
        return TimeManipulationChecker { 
            check_for_clock_lib: false, 
            time_manipulation_span: rustc_span::DUMMY_SP
        }
    }

    /// Check if the bad randomness happens. The bad randomness will possibly happens if 
    /// ``solana_program::sysvar::clock::Clock`` is used
    pub fn check(&self) -> bool {
        return self.check_for_clock_lib;
    }
}

// Hold states for the numerical precision error
pub struct NumericalPrecisionErrorChecker {
    // Check if the round function used to round up a number
    pub check_for_round_func: bool,
    // The span contains codes related to numerical precision error
    pub numerical_precision_error_span: Span,
}

impl NumericalPrecisionErrorChecker {
    pub fn new() -> NumericalPrecisionErrorChecker {
        return NumericalPrecisionErrorChecker {
            check_for_round_func: false,
            numerical_precision_error_span: rustc_span::DUMMY_SP
        }
    }

    /// Check if the numerical precision error happens. The numerical precision error will 
    /// possibly happens if ``round`` function owned by ``float`` data type is used
    pub fn check(&self) -> bool {
        return self.check_for_round_func;
    }
}





