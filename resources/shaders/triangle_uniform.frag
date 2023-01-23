#version 450

layout (location = 0) in vec3 fragColor;
layout (location = 1) in vec2 fragTexCoord;

layout (location = 0) out vec4 outColor;

// https://github.com/gfx-rs/naga/issues/1012
//layout (binding = 1) uniform sampler2D texSampler;

layout (binding = 1) uniform texture2D fragTexture;
layout (binding = 2) uniform sampler fragSampler;

void main() {
    outColor = vec4(fragColor, 1.0) * texture(sampler2D(fragTexture, fragSampler), fragTexCoord);
}
