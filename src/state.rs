use crate::config::Config;

#[derive(Debug)]
pub struct Interaction {
    pub debug_thoughts: String,
    pub file_changes: String,
    pub build_output: String,
}

pub struct History {
    config: Config,
    interactions: Vec<Interaction>,
}

impl History {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            interactions: Vec::new(),
        }
    }

    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }

    pub fn build_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str(&self.config.system_prompt);
        prompt.push_str("\n\n");
        prompt.push_str(&self.config.basic_query);
        prompt.push_str("\n\n");
        prompt.push_str(&self.config.codebase_context);

        if self.interactions.is_empty() {
            return prompt;
        }

        let first_interaction = &self.interactions[0];
        prompt.push_str(
            "\n\nThe above query was provided, and you provided the following data in your response:\n",
        );
        prompt.push_str(&first_interaction.debug_thoughts);
        prompt.push('\n');
        prompt.push_str(&first_interaction.file_changes);
        prompt.push_str(
            "\nWhen the file replacements were made, the build provided the following output:\n",
        );
        prompt.push_str(&first_interaction.build_output);

        for interaction in self.interactions.iter().skip(1) {
            prompt.push_str("\n\nYou then provided the following data in your subsequent response:\n");
            prompt.push_str(&interaction.debug_thoughts);
            prompt.push('\n');
            prompt.push_str(&interaction.file_changes);
            prompt.push_str(
                "\nWhen the file replacements were made, the build provided the following output:\n",
            );
            prompt.push_str(&interaction.build_output);
        }

        prompt
    }
}