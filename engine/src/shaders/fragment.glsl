precision mediump float;

varying vec4 vFragColor;
varying vec2 vTexCoord;

uniform sampler2D uSampler;

void main() {
  gl_FragColor = texture2D(uSampler, vTexCoord) * vFragColor;
  gl_FragColor.rgb *= gl_FragColor.a;
  if (gl_FragColor.a < 0.5) discard;
  // gl_FragColor = vec4(vTexCoord, 0.0, 1.0);
}