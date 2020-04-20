precision highp float;

varying vec4 vFragColor;
varying vec2 vTexCoord;
varying vec2 vPosition;

uniform vec2 uFirePos;
uniform float uFireHeat;
uniform sampler2D uSampler;

const float c = 500.0;

const float LuminancePreservationFactor = 1.0;

const float PI2 = 6.2831853071;

// Valid from 1000 to 40000 K (and additionally 0 for pure full white)
vec3 colorTemperatureToRGB(const in float temperature){
  // Values from: http://blenderartists.org/forum/showthread.php?270332-OSL-Goodness&p=2268693&viewfull=1#post2268693   
  mat3 m = (temperature <= 6500.0) ? mat3(vec3(0.0, -2902.1955373783176, -8257.7997278925690),
	                                      vec3(0.0, 1669.5803561666639, 2575.2827530017594),
	                                      vec3(1.0, 1.3302673723350029, 1.8993753891711275)) : 
	 								 mat3(vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
   	                                      vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
	                                      vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275)); 
  return mix(clamp(vec3(m[0] / (vec3(clamp(temperature, 1000.0, 40000.0)) + m[1]) + m[2]), vec3(0.0), vec3(1.0)), vec3(1.0), smoothstep(1000.0, 0.0, temperature));
}


void main() {
  vec4 texColor = texture2D(uSampler, vTexCoord) * vFragColor;
  texColor.rgb *= texColor.a;
  if (texColor.a < 0.5) discard;

  float r = length(vPosition - uFirePos);
  float r2 = (((max(0.0, r) - 48.0) + c) / c);
  r2 = r2 * r2;
  float temperature = clamp(10000.0 - (((uFireHeat) * 10000.0) / r2), 4000.0, 10000.0);

  vec3 inColor = texColor.rgb;

  vec3 tempRGB = colorTemperatureToRGB(temperature);
  vec3 outColor = mix(inColor, inColor * tempRGB, 0.5); 
  outColor = mix(outColor, tempRGB, 0.1);

  // luminance
  outColor.rgb *= clamp(uFireHeat/r2, 0.2, 1.0);

  gl_FragColor = vec4(outColor, texColor.a);
}