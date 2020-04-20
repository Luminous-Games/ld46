precision highp float;

attribute vec3 aPosition;
attribute vec3 aColor;
attribute vec2 aTexCoord;

varying vec4 vFragColor;
varying vec2 vTexCoord;
varying vec2 vPosition;

uniform mat4 uViewport;
uniform mat4 uTransform;

void main() {
  vPosition = aPosition.xy;
  gl_Position = uViewport * uTransform * vec4(aPosition, 1.0);
  vFragColor = vec4(aColor, 1.0);
  vTexCoord = aTexCoord;
}