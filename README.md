# slang_struct
Often times when writing graphics code in c++ it's very nice to share struct definitions in a common file that's included both in shaders and host code. Slang has compatiable enough syntax for structs and preprocessor macros with c++ to allow this to happen seamlessly but for Rust you usually have to just duplicate the struct definitions. This crate aims to provides some procedual macros to make this process possible.

# Features
The first macro this crate provides is `slang_struct`, this macro takes a variant number of structs written in Slang style and converts them to rust style structs.

For example:
```rust
slang_struct! {
  struct Example
  {
    float3 foo;
    uint32_t bar;
  }
}
```
Creates this struct in place
```rust
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Example {
  foo: [f32; 3],
  bar: u32
}
```

Currently the built in types it converts are `int8_t`, `uint8_t`, `int16_t`, `uint16_t`, `int32_t`, `uint32_t`, `int`, `uint`, `int64_t`, `uint64_t`, `float`, `float2`, `float3`, `float4`, and `float4x4`. Pointer types such as `float3*` are converted to `u64`.

You can also of course use other structs but they must be provided in some way, either directly in the same macro or seperate definitons in Rust and Slang side. Notable this macro currently only works with struct definitions one after the other, if theres any other code (including macros, method definitions, or visibility modifiers) then this macro won't work. Potentially some of this syntax will be allowed in the future but not currenlty.

The other potentially more useful macro is `slang_include` which takes the content of a file relative to the working directly and passes it to `slang_struct`

So if you have a file called `include.inl` with the contents of
```cpp
struct Vertex
{
  float3 position;
  float2 uv;
}

struct DrawPush
{
  Vertex* vertices;
  float4x4 viewProj;
}
```
And you do `slang_include!("include.inl")` in rust then it'll put
```rust
#[repr(C)]
#[derive(Copy, Clone, Default]
struct Vertex {
  position: [f32; 3]
  uv: [f32; 2]
};

#[repr(C)]
#[derive(Copy, Clone, Default]
struct DrawPush {
  vertices: u64,
  viewProj: [f32; 16]
};
```

# Optional features
* `glam` - Converts vector and matrix types to glam types instead of float arrays (e.g. `float3` becomes `glam::Vec3`).
* `bytemuck` - Derives `bytemuck::Pod` and `bytemuck::Zeroable` for structs made by this macro.

Note: This crate does not include those packages by itself for the sake of scalability. You must add these dependencies seperately.
