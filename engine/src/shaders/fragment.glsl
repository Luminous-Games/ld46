precision highp float;

varying vec4 vFragColor;
varying vec2 vTexCoord;

uniform sampler2D uSampler;

void main() {
  gl_FragColor = texture2D(uSampler, vTexCoord) * vFragColor;
  gl_FragColor.rgb *= gl_FragColor.a;
  // gl_FragColor = vec4(0.0, vTexCoord, 1.0);
  if (gl_FragColor.a < 0.4) discard;
}