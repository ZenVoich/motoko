var a = 'N';

func foo() = if (a == 'Y') {} else {};
func barX() = if (a == 'Y') {} else {};

barX();
foo();

// CHECK: func $foo
// CHECK: i32.eq
// CHECK-NEXT: drop
// CHECK: func $barX

//SKIP run
//SKIP run-ir
//SKIP run-low
