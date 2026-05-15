use anyhow::{Context, anyhow};
use cntp_config::ContemporaryConfig;
use std::fs::File;
use std::io::BufReader;
use std::iter;
use std::path::Path;
use tracing::{debug, info};
use xml::attribute::Attribute;
use xml::name::Name;
use xml::reader::XmlEvent;
use xml::{EmitterConfig, EventReader, writer};

const CNTP_RS_NAMESPACE: &str = "https://vicr123.com/cntp-rs";

pub fn copy_appstream_metainfo(
    source: &Path,
    output: &Path,
    contemporary_config: &ContemporaryConfig,
) -> anyhow::Result<()> {
    let available_localisations = {
        let mut available_localisations = contemporary_config.available_localisations();
        available_localisations.sort();
        available_localisations
    };

    let input_file = File::open(source)
        .with_context(|| format!("Failed to open source file: {}", source.display()))?;
    let input_file = BufReader::new(input_file);

    let output_file = File::create(output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;
    let mut xml_writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(output_file);

    let input_reader = EventReader::new(input_file);
    let mut off_depth = 0_u32;
    let mut start_event = None;
    for e in input_reader {
        info!("Processing XML event: {:?}", e);
        let e = e?;
        match &e {
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                let mut attributes = attributes.clone();
                let tr_key = attributes
                    .iter()
                    .position(|attr| {
                        attr.name.local_name == "trkey"
                            && attr
                                .name
                                .namespace
                                .as_ref()
                                .is_some_and(|namespace| namespace == CNTP_RS_NAMESPACE)
                    })
                    .map(|pos| attributes.remove(pos));

                // Remove xmlns:cntp from the output
                let mut namespace = namespace.clone();
                namespace.0.retain(|_, value| value != CNTP_RS_NAMESPACE);

                if off_depth > 0 {
                    if tr_key.is_some() {
                        return Err(anyhow!("Found nested tr attributes"));
                    }

                    off_depth = off_depth.strict_add(1);
                } else if let Some(tr_key) = tr_key {
                    // Save the event
                    start_event = Some((
                        name.clone(),
                        attributes.clone(),
                        namespace.clone(),
                        tr_key,
                    ));

                    // Stop writing until the corresponding EndElement
                    off_depth = 1;
                } else {
                    // Write this event as normal
                }

                xml_writer.write(writer::XmlEvent::StartElement {
                    name: name.borrow(),
                    attributes: attributes.iter().map(|a| a.borrow()).collect(),
                    namespace: namespace.borrow(),
                })?;
            }
            XmlEvent::EndElement { .. } => {
                if let Some(e) = e.as_writer_event() {
                    xml_writer.write(e)?;
                }

                if off_depth > 0 {
                    off_depth = off_depth.strict_sub(1);
                    if off_depth == 0 {
                        let (name, attributes, namespace, tr_key) = start_event
                            .take()
                            .with_context(|| "Start event not found")?;

                        for lang in &available_localisations {
                            let Some(translation) =
                                contemporary_config.lookup_translation(lang, &tr_key.value)
                            else {
                                // Skip this node if the translation is missing
                                continue;
                            };

                            let lang_attribute =
                                Attribute::new(Name::prefixed("xml", "lang"), lang);

                            xml_writer.write(writer::XmlEvent::StartElement {
                                name: name.borrow(),
                                attributes: iter::once(lang_attribute)
                                    .chain(attributes.iter().map(|a| a.borrow()))
                                    .collect(),
                                namespace: namespace.borrow(),
                            })?;
                            xml_writer.write(writer::XmlEvent::Characters(translation))?;
                            xml_writer.write(writer::XmlEvent::EndElement { name: None })?;
                        }
                    }
                }
            }
            _ => {
                if let Some(e) = e.as_writer_event() {
                    xml_writer.write(e)?;
                }
            }
        }
    }

    Ok(())
}
