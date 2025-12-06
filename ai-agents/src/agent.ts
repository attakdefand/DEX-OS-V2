import OpenAI from "openai";
import dotenv from "dotenv";

dotenv.config();

const openai = new OpenAI({
    apiKey: process.env.OPENAI_API_KEY,
});

async function main() {
    if (!process.env.OPENAI_API_KEY) {
        console.error("Please set OPENAI_API_KEY in .env file");
        return;
    }

    console.log("Creating a simple assistant...");

    try {
        const completion = await openai.chat.completions.create({
            messages: [{ role: "system", content: "You are a helpful assistant for DEX-OS." }, { role: "user", content: "Hello, what can you do?" }],
            model: "gpt-3.5-turbo",
        });

        console.log("Agent response:", completion.choices[0].message.content);
    } catch (error) {
        console.error("Error creating chat completion:", error);
    }
}

main();
