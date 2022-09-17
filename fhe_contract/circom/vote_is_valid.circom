pragma circom 2.0.0;

include "./circomlib/circuits/comparators.circom";

template vote_is_valid() {  

   // this is our vote
   signal input vote[10];  
   signal output c;  

   var sum = 0;
   for(var i = 0; i < 10; i++){
      sum += vote[i];      
   }

   // this is a simplification, we could be looking at each component and ensuring
   // it is either 0 or 1
   component is_less_than = LessEqThan(10);
   is_less_than.in[0] <== sum;
   is_less_than.in[1] <== 1;
   signal o;
   is_less_than.out ==> o;
   log("The result is ", o);
   
   component is_more_than = GreaterEqThan(10);
   is_more_than.in[0] <== sum;
   is_more_than.in[1] <== 0;
   signal o1;
   is_more_than.out ==> o1;
   log("The result is ", o1);

   component f_equal = ForceEqualIfEnabled();
   f_equal.enabled <== 1;
   f_equal.in[0] <== o*o1;
   f_equal.in[1] <== 1;
   signal o2;
   is_more_than.out ==> o2;
   log("The result to forced equal is ", o2);


   // Constraints.  
   c <== o * o1;  
}

// for more circuits - https://github.com/iden3/circomlib
component main= vote_is_valid();