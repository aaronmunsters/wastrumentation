pub(crate) trait AsOption {
    fn as_option(&self) -> &str;
}

macro_rules! option {
    ($name: ident, $option_str: literal) => {
        #[derive(Debug, Default)]
        pub enum $name {
            Enable,
            #[default]
            Disable,
        }

        impl AsOption for $name {
            fn as_option(&self) -> &str {
                match self {
                    &Self::Enable => $option_str,
                    &Self::Disable => "",
                }
            }
        }
    };
}

// Disables validation, assumes inputs are correct
option!(NoValidate, "--no-validation");
// Rename exports to avoid conflicts (rather than error)
option!(RenameExportConflicts, "--rename-export-conflicts");

// FEATURES
// sign extension operations
option!(SignExt, "--enable-sign-ext");
// atomic operations
option!(Threads, "--enable-threads");
// mutable globals
option!(MutableGlobals, "--enable-mutable-globals");
// nontrapping float-to-int
option!(NontrappingFloatToInt, "--enable-nontrapping-float-to-int");
// SIMD operations and types
option!(Simd, "--enable-simd");
// bulk memory operations
option!(BulkMemory, "--enable-bulk-memory");
// memory.copy and memory.fill
option!(BulkMemoryOpt, "--enable-bulk-memory-opt");
// LEB encoding of call-indirect
option!(CallIndirectOverlong, "--enable-call-indirect-overlong");
// exception handling operations
option!(ExceptionHandling, "--enable-exception-handling");
// tail call operations
option!(TailCall, "--enable-tail-call");
// reference types
option!(ReferenceTypes, "--enable-reference-types");
// multivalue functions
option!(Multivalue, "--enable-multivalue");
// garbage collection
option!(Gc, "--enable-gc");
// memory64
option!(Memory64, "--enable-memory64");
// relaxed SIMD
option!(RelaxedSimd, "--enable-relaxed-simd");
// extended const expressions
option!(ExtendedConst, "--enable-extended-const");
// strings
option!(Strings, "--enable-strings");
// multimemory
option!(Multimemory, "--enable-multimemory");
// typed continuations
option!(TypedContinuations, "--enable-typed-continuations");
// shared-everything threads
option!(SharedEverything, "--enable-shared-everything");
// float 16 operations
option!(Fp16, "--enable-fp16");
