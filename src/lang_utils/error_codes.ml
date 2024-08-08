(* A list of all error codes used in the compiler. The second field in
   each tuple represents a possible long-form description of the
   error. *)
let error_codes : (string * string option) list =
  [
    "M0000", None; (* Internal compiler error *)
    "M0001", None; (* Parsing errors *)
    "M0002", None; (* Lexer errors *)
    "M0003", Some([%blob "lang_utils/error_codes/M0003.md"]); (* Self-import *)
    "M0004", None; (* IDL file doesn't define a service *)
    "M0005", None; (* Case mismatch between import and filename *)
    "M0006", None; (* Failed to parse import URL *)
    "M0007", None; (* Failed to parse actor alias *)
    "M0008", None; (* Actor import without IDL path *)
    "M0009", None; (* File not found for import *)
    "M0010", None; (* Imported package was not defined *)
    "M0011", None; (* Imported canister alias was not defined *)
    "M0012", None; (* File for package does not exist *)
    "M0013", None; (* Tried to map the prim package *)
    "M0014", None; (* Non-static expression in library or module *)
    "M0015", None; (* Non-trivial pattern in static expression *)
    "M0016", None; (* Usage before definition *)
    "M0017", None; (* Duplicate binding in pattern *)
    "M0018", None; (* Duplicate name in type *)
    "M0019", None; (* Colliding hashes in type *)
    "M0020", None; (* Unresolved import *)
    "M0021", None; (* Cannot infer type of forward import *)
    "M0022", None; (* Imported file not loaded *)
    "M0023", None; (* Path expression is not a module, object, or actor *)
    "M0024", None; (* Cannot infer type of forward variable *)
    "M0025", None; (* Unavailable variable *)
    "M0026", None; (* Unbound variable *)
    "M0027", None; (* Cannot infer type of forward field reference *)
    "M0028", None; (* Field does not exist in type *)
    "M0029", None; (* Unbound type *)
    "M0030", None; (* Type field does not exist in type *)
    "M0031", None; (* Shared function has non-shared parameter type *)
    "M0032", None; (* Shared function has non-shared return type *)
    "M0033", None; (* Async has non-shared content type *)
    "M0034", None; (* Shared constructor has non-shared parameter type *)
    "M0035", None; (* Invalid return type for shared function *)
    "M0036", None; (* Invalid return type for shared query function *)
    "M0037", Some([%blob "lang_utils/error_codes/M0037.md"]); (* Misplaced async expression *)
    "M0038", None; (* Misplaced await *)
    "M0039", None; (* Misplaced try/throw/catch *)
    "M0040", None; (* Unknown primitive type *)
    "M0041", None; (* Shared function has non-async result type *)
    "M0042", None; (* Actor field must have shared function type *)
    "M0043", None; (* Type parameter has cyclic bounds *)
    "M0044", None; (* Duplicate name in type parameter list *)
    "M0045", None; (* Wrong number of type arguments *)
    "M0046", None; (* Type argument does not match parameter bound *)
    "M0047", None; (* Send capability required *)
    "M0048", None; (* Literal out of range *)
    "M0049", None; (* String literal is not valid utf8 *)
    "M0050", None; (* Literal does not have expected type *)
    "M0051", None; (* Duplicate definition in block *)
    (* "M0052" DEFUNCT Duplicate definition in object *)
    "M0053", None; (* Cannot infer expression in class using forward type reference *)
    "M0054", None; (* Cannot infer type of primitive *)
    "M0055", None; (* Cannot infer type of forward variable *)
    "M0056", None; (* Variable is not available in compiled code *)
    "M0057", None; (* Unbound variable *)
    "M0058", None; (* No type can be inferred for actor reference *)
    "M0059", None; (* Operator is not defined for operand type *)
    "M0060", None; (* Operator is not defined for operand types *)
    "M0061", None; (* Comparing abstract type to itself at supertype *)
    "M0062", None; (* Comparing incompatible type at common supertype *)
    "M0063", None; (* Show is not defined for operand type *)
    "M0064", None; (* Misplaced '!' without enclosing do block *)
    "M0065", None; (* Expected option type before '!' *)
    "M0066", None; (* Tuple projection is out of bounds *)
    "M0067", None; (* Expected tuple type *)
    "M0068", None; (* Actors are not supported *)
    "M0069", None; (* Non-toplevel actor *)
    "M0070", None; (* Expected object type *)
    "M0071", None; (* Cannot infer type of forward field reference *)
    "M0072", None; (* Field does not exist in type *)
    "M0073", None; (* Expected mutable assignment target *)
    "M0074", None; (* Array elements have inconsistent types *)
    "M0075", None; (* Expected array type *)
    "M0076", None; (* Shared functions are not supported *)
    "M0077", None; (* Shared function is only allowed as a public field of an actor *)
    "M0078", None; (* Shared function with () result type has unexpected body *)
    "M0079", None; (* Shared function with async result type has non-async body *)
    (* "M0080" DEFUNCT Local class type is contained in inferred block type *)
    "M0081", None; (* If branches have inconsistent types *)
    "M0082", None; (* Expected iterable type *)
    "M0083", None; (* Unbound label *)
    "M0084", None; (* Cannot infer return type *)
    "M0085", None; (* Misplaced return *)
    "M0086", None; (* Async expressions are not supported *)
    "M0087", None; (* Ill-scoped await *)
    "M0088", None; (* Expected async type *)
    "M0089", None; (* Redundant ignore *)
    "M0090", None; (* Actor reference must have an actor type *)
    "M0091", None; (* Mutable array expression cannot produce expected type *)
    "M0092", None; (* Async cannot produce scope *)
    "M0093", None; (* Bases of record extensions must be objects or modules *)
    "M0094", None; (* Shared functions does not match shared function type *)
    "M0095", None; (* Function return type does not match expected return type *)
    "M0096", None; (* Expression can't produce expected type *)
    "M0097", None; (* Expected function type *)
    "M0098", None; (* Cannot instantiate function type *)
    "M0099", None; (* Shared function argument contains abstract type *)
    "M0100", None; (* Shared function call result contains abstract type *)
    "M0101", None; (* Switch with inconsistent branch types *)
    "M0102", None; (* Cannot infer type of wildcard *)
    "M0103", None; (* Cannot infer type of variable *)
    (* "M0104" DEFUNCT Pattern branches have incompatible types *)
    (* "M0105" DEFUNCT Variables are not allowed in pattern alternatives *)
    "M0106", None; (* Shared function cannot take a context pattern *)
    "M0107", None; (* Non-actor class cannot take a context pattern *)
    "M0108", None; (* Actor class cannot take a context pattern *)
    "M0109", None; (* Class cannot be a query *)
    "M0110", None; (* Literal pattern cannot consume type *)
    "M0111", None; (* Operator pattern cannot consume type *)
    "M0112", None; (* Tuple pattern cannot consume type *)
    "M0113", None; (* Object pattern cannot consume type *)
    "M0114", None; (* Object pattern cannot consume actor type *)
    "M0115", None; (* Option pattern cannot consume type *)
    "M0116", None; (* Variant pattern cannot consume type *)
    "M0117", None; (* Pattern cannot consume type *)
    "M0118", None; (* Tuple pattern size mismatch *)
    "M0119", None; (* Object field is not contained in type *)
    "M0120", None; (* Cannot pattern match mutable field *)
    "M0121", None; (* Duplicate field in object pattern *)
    "M0122", None; (* Colliding hashes in object type *)
    (* "M0123" DEFUNCT Local class type contained in object or actor type *)
    "M0124", None; (* Public actor field has non-shared type *)
    "M0125", None; (* Public actor field needs to be a manifest function *)
    "M0126", None; (* Shared function cannot be private *)
    "M0127", None; (* System function with wrong type *)
    "M0128", None; (* Function with system function name but wrong visibility *)
    "M0129", None; (* Unexpected system method name *)
    "M0130", None; (* Misplaced system visibility *)
    "M0131", None; (* Expected stable type *)
    "M0132", None; (* Misplaced stability declaration *)
    "M0133", None; (* Misplaced stability modifier *)
    "M0134", None; (* Class body type mismatch *)
    "M0135", None; (* Actor class has non-async return type *)
    "M0136", None; (* Empty block type mismatch *)
    "M0137", Some([%blob "lang_utils/error_codes/M0137.md"]); (* Type definition references type parameter from outer scope *)
    "M0138", None; (* Actor classes are not supported *)
    "M0139", None; (* Inner actor classes are not supported *)
    "M0140", None; (* Actor classes with type parameters are not supported *)
    "M0141", None; (* Forbidden declaration in program *)
    "M0142", None; (* An imported library should be a module or named actor class *)
    "M0143", None; (* Imported actor class cannot be anonymous *)
    "M0144", None; (* Expected a module or actor class *)
    "M0145", None; (* Pattern does not cover value *)
    "M0146", None; (* Pattern is never matched *)
    (* "M0147" DEFUNCT Object syntax is deprecated in this position *)
    (* "M0148" DEFUNCT Block syntax is deprecated in this position *)
    "M0149", Some([%blob "lang_utils/error_codes/M0149.md"]); (* Expected mutable 'var' field, found immutable field *)
    "M0150", Some([%blob "lang_utils/error_codes/M0150.md"]); (* Expected immutable field, found mutable 'var' field *)
    "M0151", Some([%blob "lang_utils/error_codes/M0151.md"]); (* missing field in object literal *)
    (* "M0152" DEFUNCT Word field deprecation *)
    "M0153", Some([%blob "lang_utils/error_codes/M0153.md"]); (* IDL types not expressible in Motoko *)
    "M0154", Some([%blob "lang_utils/error_codes/M0154.md"]); (* Deprecation annotation *)
    "M0155", Some([%blob "lang_utils/error_codes/M0155.md"]); (* Inferred type Nat for subtraction *)
    "M0156", Some([%blob "lang_utils/error_codes/M0156.md"]); (* block contains expansive type definitions *)
    "M0157", Some([%blob "lang_utils/error_codes/M0157.md"]); (* block contains non-productive type definitions *)
    "M0158", Some([%blob "lang_utils/error_codes/M0158.md"]); (* a public class cannot be anonymous, please provide a name *)
    (* "M0159" DEFUNCT Word deprecation *)
    "M0160", None; (* Candid method name is not a valid Motoko identifier *)
    "M0161", None; (* Candid float32 type cannot be imported as a Motoko type *)
    "M0162", None; (* Candid service constructor type not supported as Motoko type *)
    (* "M0163" DEFUNCT Cannot import a Candid service constructor *)
    "M0164", None; (* Unknown record or variant label in textual representation *)
    "M0165", None; (* Odd expected type *)
    "M0166", None; (* Type intersection results in abstract type *)
    "M0167", None; (* Type union results in bottom type *)
    "M0168", None; (* Type union or intersection on forward types *)
    "M0169", None; (* Stable variable cannot be discarded *)
    "M0170", None; (* Stable variable must subtype *)
    "M0171", None; (* Stable variable changing mutability *)
    "M0172", None; (* to_candid produces Blob, not -- *)
    "M0173", None; (* to_candid arg must have shared type, not -- *)
    "M0174", None; (* from_candid produces an optional shared, not -- *)
    "M0175", None; (* to_candid argument must have shared type, not -- *)
    "M0176", None; (* from_candid requires known type from context *)
    "M0177", None; (* Bases of record extensions must not have common fields that are not overwritten *)
    "M0178", None; (* Bases of record extensions must be either objects or modules *)
    "M0179", None; (* Mutable (var) fields from bases must be overwritten explicitly *)
    "M0180", None; (* Shared function has unexpected type parameters *)
    "M0181", None; (* Verification mode assertions not allowed *)
    "M0182", None; (* Timer defined by user when -no-timer flag is specified *)
    "M0183", None; (* Incompatible async sorts *)
    "M0184", None; (* Cannot infer or-pattern, please use an annotation *)
    "M0185", None; (* Importing Candid service constructor as instantiated service *)
    "M0186", None; (* Send capability required (calling shared from composite) *)
    "M0187", None; (* Send capability required (calling composite from non-composite) *)
    "M0188", None; (* Send capability required (calling shared from query) *)
    "M0189", None; (* Different set of bindings in pattern alternatives *)
    "M0190", None; (* Types inconsistent for alternative pattern variables, losing information *)
    "M0191", None; (* Code requires Wasm features ... to execute *)
    "M0192", None; (* Object/Actor/Module body type mismatch *)
    "M0193", None; (* Can't declare actor class to have `async*` result *)
    "M0194", Some([%blob "lang_utils/error_codes/M0194.md"]); (* Unused identifier warning *)
    "M0195", Some([%blob "lang_utils/error_codes/M0195.md"]); (* warn that `system` capability is implicitly supplied *)
    "M0196", None; (* `system` capability supplied but not required *)
    "M0197", Some([%blob "lang_utils/error_codes/M0197.md"]); (* `system` capability required *)
    "M0198", Some([%blob "lang_utils/error_codes/M0198.md"]); (* Unused field pattern warning *)
    "M0199", Some([%blob "lang_utils/error_codes/M0199.md"]); (* Deprecate experimental stable memory *)
  ]
