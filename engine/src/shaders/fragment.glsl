precision mediump float;

varying vec4 vFragColor;
varying vec2 vTexCoord;

uniform sampler2D uSampler;

void main() {
  gl_FragColor = texture2D(uSampler, vTexCoord) * vFragColor;
  gl_FragColor = texture2D(uSampler, vTexCoord);
  // gl_FragColor = vec4(vTexCoord, 0.0, 1.0);
}