{{#each defs.provides as |provide|}}
{{#if provide.described}}
#[derive(Clone, Debug, PartialEq)]
pub enum {{provide.name}} {
{{#each provide.options as |option|}}
    {{option.ty}}({{option.ty}}),
{{/each}}
}
impl DecodeFormatted for {{provide.name}} {
    fn decode_with_format(input: &[u8], fmt: u8) -> Result<(&[u8], Self)> {
        let (input, descriptor) = Descriptor::decode_with_format(input, fmt)?;
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
impl Encode for {{provide.name}} {
    fn encoded_size(&self) -> usize {
        match *self {
            {{#each provide.options as |option|}}
            {{provide.name}}::{{option.ty}}(v) => v.encoded_size(),
            {{/each}}
        }
    }
    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            {{#each provide.options as |option|}}
            {{provide.name}}::{{option.ty}}(v) => v.encode(buf),
            {{/each}}
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
{{#if enum.is_symbol}}
impl {{enum.name}} {
    pub fn try_from(v: &Symbol) -> Result<Self> {
        match v.as_str() {
            {{#each enum.items as |item|}}
            "{{item.value}}" => Ok({{enum.name}}::{{item.name}}),
            {{/each}}
            _ => Err("unknown {{enum.name}} option.".into())
        }
    }
}
impl DecodeFormatted for {{enum.name}} {
    fn decode_with_format(input: &[u8], fmt: u8) -> Result<(&[u8], Self)> {
        let (input, base) = Symbol::decode_with_format(input, fmt)?;
        Ok((input, Self::try_from(&base)?))
    }
}
impl Encode for {{enum.name}} {
    fn encoded_size(&self) -> usize {
        match *self {
            {{#each enum.items as |item|}}
            {{enum.name}}::{{item.name}} => "{{item.value}}".encoded_size(),
            {{/each}}
        }
    }
    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            {{#each enum.items as |item|}}
            {{enum.name}}::{{item.name}} => Symbol::from_static("{{item.value}}").encode(buf),
            {{/each}}
        }
    }
}
{{else}}
impl {{enum.name}} {
    pub fn try_from(v: {{enum.ty}}) -> Result<Self> {
        match v {
            {{#each enum.items as |item|}}
            {{item.value}} => Ok({{enum.name}}::{{item.name}}),
            {{/each}}
            _ => Err("unknown {{enum.name}} option.".into())
        }
    }
}
impl DecodeFormatted for {{enum.name}} {
    fn decode_with_format(input: &[u8], fmt: u8) -> Result<(&[u8], Self)> {
        let (input, base) = {{enum.ty}}::decode_with_format(input, fmt)?;
        Ok((input, Self::try_from(base)?))
    }
}
impl Encode for {{enum.name}} {
    fn encoded_size(&self) -> usize {
        match *self {
            {{#each enum.items as |item|}}
            {{item.name}} => {{item.value}}.encoded_size(),
            {{/each}}
        }
    }
    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            {{#each enum.items as |item|}}
            {{item.name}} => {{item.value}}.encode(buf),
            {{/each}}
        }
    }
}
{{/if}}
{{/each}}

{{#each defs.lists as |list|}}
#[derive(Clone, Debug, PartialEq)]
pub struct {{list.name}} {
    {{#each list.fields as |field|}}
    {{#if field.optional}}
    {{field.name}}: Option<{{{field.ty}}}>,
    {{else}}
    {{field.name}}: {{{field.ty}}},
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
                    pub fn {{field.name}}(&self) -> Option<&{{{field.ty}}}> { self.{{field.name}}.as_ref() }
                {{else}}
                    pub fn {{field.name}}(&self) -> &{{{field.ty}}} { &self.{{field.name}} }
                {{/if}}
            {{else}}
                {{#if field.optional}}
                    pub fn {{field.name}}(&self) -> Option<{{{field.ty}}}> { self.{{field.name}} }
                {{else}}
                    pub fn {{field.name}}(&self) -> {{{field.ty}}} { self.{{field.name}} }
                {{/if}}
            {{/if}}
        {{/if}}
    {{/each}}
}

impl DecodeFormatted for {{list.name}} {
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self)> {
        let (input, header) = decode_list_header(input, format)?;
        let mut count = header.count;
        let mut input = input;
        {{#each list.fields as |field|}}
        {{#if field.optional}}
        let {{field.name}}: Option<{{{field.ty}}}>;
        if count > 0 {
            let decoded = Option::<{{{field.ty}}}>::decode(input)?;
            input = decoded.0;
            {{field.name}} = decoded.1;
            count -= 1;
        }
        else {
           {{field.name}} = None;
        }
        {{else}}
        let {{field.name}}: {{{field.ty}}};
        if count > 0 {
            {{#if field.default}}
            let (in1, decoded) = Option::<{{{field.ty}}}>::decode(input)?;
            {{field.name}} = decoded.unwrap_or({{field.default}});
            {{else}}
            let (in1, decoded) = {{{field.ty}}}::decode(input)?;
            {{field.name}} = decoded;
            {{/if}}
            input = in1;
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
        Ok((input, {{list.name}} {
        {{#each list.fields as |field|}}
        {{field.name}},
        {{/each}}
        }))
    }
}

impl Encode for {{list.name}} {
    fn encoded_size(&self) -> usize {
        3 // 0x00 0x53 <descriptor code>
        {{#each list.fields as |field|}}
        + self.{{field.name}}.encoded_size()
        {{/each}}
    }

    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(codec::FORMATCODE_DESCRIBED);
        {{list.descriptor.code}}.encode(buf);
    }
}

{{/each}}