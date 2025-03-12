import amqp from "amqplib/callback_api.js";
import { QUEUE_NAME, RABBITMQ_URL } from "../config/app-config.js";
import type { MessageData } from "../models/message.js";

export interface QueueMessage {
  from: string;
  data: MessageData;
}

export type MessageHandler = (
  message: QueueMessage,
  ack: () => void,
  nack: () => void
) => Promise<void>;

export function setupQueueConsumer(handler: MessageHandler): Promise<void> {
  return new Promise((resolve, reject) => {
    amqp.connect(RABBITMQ_URL, function (error0, connection) {
      if (error0) {
        reject(error0);
        return;
      }

      connection.createChannel(function (error1, channel) {
        if (error1) {
          reject(error1);
          return;
        }

        channel.assertQueue(QUEUE_NAME, {
          durable: false,
        });

        console.log(" [*] Waiting for %s. To exit press CTRL+C", QUEUE_NAME);

        channel.consume(
          QUEUE_NAME,
          async function (msg) {
            if (!msg) return;

            try {
              const content = JSON.parse(msg.content.toString());
              await handler(
                content,
                () => channel.ack(msg),
                () => channel.nack(msg, false, false)
              );
            } catch (error) {
              console.error(" [-] Error processing message:", error);
              channel.nack(msg, false, false);
            }
          },
          { noAck: false }
        );

        resolve();
      });
    });
  });
}
