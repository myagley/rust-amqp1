{{#each defs.provides as |provide|}}
{{#if provide.described}}
#[derive(Clone, Debug, PartialEq)]
pub enum {{provide.name}} {
{{#each provide.options as |item|}}
    {{item.ty}}({{item.ty}}),
{{/each}}
}

impl Decode for {{provide.name}} {
    fn decode(input: &[u8]) -> Result<(&[u8], Self)> {
        let (input, descriptor) = Descriptor::decode(input)?;
        match descriptor {
            {{#each provide.options as |option|}}
            Descriptor::Ulong({{option.descriptor.code}}) => {{option.ty}}::decode(input).map(|(i, r)| (i, {{provide.name}}::{{option.ty}}(r))),
            {{/each}}
            {{#each provide.options as |option|}}
            Descriptor::Symbol(ref a) if a.as_str() == "{{option.descriptor.name}}" => {{option.ty}}::decode(input).map(|(i, r)| (i, {{provide.name}}::{{option.ty}}(r))),
            {{/each}}
            _ => Err(ErrorKind::Custom(codec::INVALID_DESCRIPTOR).into())
        }
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
    fn decode(input: &[u8]) -> Result<(&[u8], Self)> {
        let (input, fmt) = decode_format_code(input)?;
        let (input, header) = decode_list_header(input, format)?;
        let mut count = header.count;
        let mut input = input;
        {{#each list.fields as |field|}}
        {{#if field.optional}}
        let {{field.name}}: Option<{{field.ty}}>;
        if count > 0 {
            let decoded = Option::<{{field.ty}}>::decode(input)?;
            input = decoded.0;
            {{field.name}} = decoded.1;
            count -= 1;
        }
        else {
           {{field.name}} = None;
        }
        {{else}}
        let {{field.name}}: {{field.ty}};
        if count > 0 {
            {{#if field.default}}
            let decoded = Option::<{{field.ty}}>::decode(input)?;
            {{field.name}} = decoded.1.unwrap_or({{field.default}});
            {{else}}
            let decoded = {{field.ty}}::decode(input)?;
            {{field.name}} = decoded.1;
            {{/if}}
            input = decoded.0;
            count -= 1;
        }
        else {
            {{#if field.default}}
            {{field.name}} = {{field.default}};
            {{else}}
            return Err("Required field {{field.name}} was omitted.".into());
            {{/if}}
        }
        {{/if}}
        {{/each}}
        Err("nope".into())
    }
}

impl Encode for {{list.name}} {
    fn encoded_size(&self) -> usize {
        0
    }

    fn encode(&self, buf: &mut BytesMut) -> () {
        buf.put_u8(codec::FORMATCODE_DESCRIBED);
        {{list.descriptor.code}}.encode(buf);
    }
}

{{/each}}