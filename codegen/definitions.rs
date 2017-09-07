{{#each defs.provides as |provide|}}
#[derive(Clone, Debug, PartialEq)]
pub enum {{provide.name}} {
{{#each provide.options as |item|}}
    {{item.ty}}({{item.ty}}),
{{/each}}
}

{{#if provide.described}}
impl Decode2 for {{provide.name}} {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], Self> {
        validate_code!(format, codec::FORMATCODE_DESCRIBED);
        do_parse!(input,
            descriptor: call!(Descriptor::decode_with_format, format) >>
            result: call!(decode_{{snake provide.name}}, descriptor) >>
            (result)
        )
    }
}
fn decode_{{snake provide.name}}(input: &[u8], descriptor: Descriptor) -> IResult<&[u8], {{provide.name}}> {
    match descriptor {
        {{#each provide.options as |option|}}
        Descriptor::Ulong({{option.descriptor.code}}) => map!(input, call!({{option.ty}}::decode), {{provide.name}}::{{option.ty}}),
        {{/each}}
        {{#each provide.options as |option|}}
        Descriptor::Symbol(ref a) if a.as_str() == "{{option.descriptor.name}}" => map!(input, call!({{option.ty}}::decode), {{provide.name}}::{{option.ty}}),
        {{/each}}
        _ => IResult::Error(error_code!(ErrorKind::Custom(codec::INVALID_DESCRIPTOR)))
    }
}
{{else}}
impl Decode for {{provide.name}} {
    fn decode(input: &[u8]) -> IResult<&[u8], Self> {
        unimplemented!();
    }
}
{{/if}}
{{/each}}

{{#each defs.aliases as |alias|}}
pub type {{alias.name}} = {{alias.source}};
{{/each}}

{{#each defs.enums as |enum|}}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum {{enum.name}} { // {{enum.ty}}
{{#each enum.items as |item|}}
    /// {{item.value}}
    {{item.name}},
{{/each}}
}
{{/each}}

{{#each defs.lists as |list|}}
#[derive(Clone, Debug, PartialEq)]
pub struct {{list.name}} {
    {{#each list.fields as |field|}}
    {{#if field.optional}}
    {{field.name}}: Option<{{field.ty}}>,
    {{else}}
    {{field.name}}: {{field.ty}},
    {{/if}}
    {{/each}}
}

impl Described for {{list.name}} {
    fn descriptor_name(&self) -> &'static str { "{{list.descriptor.name}}" }
    fn descriptor_domain(&self) -> u32 { {{list.descriptor.domain}} }
    fn descriptor_code(&self) -> u32 { {{list.descriptor.code}} }
}

impl {{list.name}} {


    {{#each list.fields as |field|}}
        {{#if field.is_str}}
            {{#if field.optional}}
                pub fn {{field.name}}(&self) -> Option<&str> {
                    match self.{{field.name}} {
                        None => None,
                        Some(ref s) => Some(s.as_str())
                    }
                }
            {{else}}
                pub fn {{field.name}}(&self) -> &str { self.{{field.name}}.as_str() }
            {{/if}}
        {{else}}
            {{#if field.is_ref}}
                {{#if field.optional}}
                    pub fn {{field.name}}(&self) -> Option<&{{field.ty}}> { self.{{field.name}}.as_ref() }
                {{else}}
                    pub fn {{field.name}}(&self) -> &{{field.ty}} { &self.{{field.name}} }
                {{/if}}
            {{else}}
                {{#if field.optional}}
                    pub fn {{field.name}}(&self) -> Option<{{field.ty}}> { self.{{field.name}} }
                {{else}}
                    pub fn {{field.name}}(&self) -> {{field.ty}} { self.{{field.name}} }
                {{/if}}
            {{/if}}
        {{/if}}
    {{/each}}
}

impl Decode for {{list.name}} {
    fn decode(bytes: &[u8]) -> IResult<&[u8], Self, u32> {
        IResult::Incomplete(Needed::Unknown)
    }
}

impl Encode for {{list.name}} {
    fn encoded_size(&self) -> usize {
        0
    }

    fn encode(&self, buf: &mut BytesMut) -> () {
        buf.put_u8(0x00);
        {{list.descriptor.code}}.encode(buf);
    }
}

{{/each}}