module wide(
  input logic[127:0] a,
  output logic[63:0] a_hi,
  output logic[63:0] a_lo,
  input logic[63:0] b_hi,
  input logic[63:0] b_lo,
  output logic[127:0] b
);

  always_comb begin
    a_hi = a[64 +: 64];
    a_lo = a[0 +: 64];

    b = {b_hi, b_lo};
  end

endmodule
