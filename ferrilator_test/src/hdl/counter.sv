module counter(
  input clk,
  input reset,
  input enable,
  output reg [7:0] value,
  output reg overflow
);

  always @(posedge clk) begin
    if (reset) begin
      value <= 0;
      overflow <= 0;
    end
    else if (enable) begin
      if (value == 255) begin
        overflow <= 1;
      end
      else begin
        overflow <= 0;
      end
      value <= value + 1;
    end
  end

endmodule
