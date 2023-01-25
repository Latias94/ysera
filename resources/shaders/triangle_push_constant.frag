#version 450

layout (location = 0) in vec3 fragColor;
layout (location = 1) in vec2 fragTexCoord;

layout (location = 0) out vec4 outColor;

// https://github.com/gfx-rs/naga/issues/1012
// layout (binding = 1) uniform sampler2D texSampler;

layout (set = 0, binding = 1) uniform texture2D fragTexture;
layout (set = 0, binding = 2) uniform sampler fragSampler;

// It seems PushContants in glsl frag shader is not supported in naga, or we can push constant with uniform buffer instead.
// Thus I use other shader compiler like glslangValidator here. see `build.rs`
layout (push_constant) uniform PushConstants {
    // first 64 bytes of push constants are occupied by the model matrix used in the vertex shader.
    layout (offset = 64) float opacity;
} pcs;

void main() {
    outColor = vec4(fragColor, 1.0) * vec4(texture(sampler2D(fragTexture, fragSampler), fragTexCoord).rgb, pcs.opacity);
}
