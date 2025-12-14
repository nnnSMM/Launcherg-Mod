void main(
	float2 pos : POSITION,
	float2 coord : TEXCOORD,
	out noperspective float2 outCoord : TEXCOORD,
	out noperspective float4 outPos : SV_POSITION
) {
	outPos = float4(pos, 0, 1);
	outCoord = coord;
}
