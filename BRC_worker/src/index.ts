import { setupQueueConsumer } from "./services/queue-service.js";
import { SubmissionProcessor } from "./processors/submission-processor.js";

console.log("BRC Worker starting up...");

const processor = new SubmissionProcessor();

setupQueueConsumer(async (message, ack, nack) => {
  try {
    console.log(" [x] Received task from", message.from);

    if (!message.from || !message.data) {
      console.error(" [-] Invalid task");
      nack();
      return;
    }

    await processor.process(message.data);
    ack();
  } catch (error) {
    console.error(" [-] Processing error:", error);
    nack();
  }
})
  .then(() => {
    console.log("Queue consumer setup complete");
  })
  .catch((error) => {
    console.error("Failed to set up queue consumer:", error);
    process.exit(1);
  });
