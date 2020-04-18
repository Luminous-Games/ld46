precision mediump float;

attribute vec2 aPosition;
attribute vec3 aColor;
attribute vec2 aTexCoord;

varying vec4 vFragColor;
varying vec2 vTexCoord;

void main() {
  gl_Position = vec4(aPosition, 0.0, 1.0);
  vFragColor = vec4(aColor, 1.0);
  vTexCoord = aTexCoord;
}