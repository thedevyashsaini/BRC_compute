// Function to delete the folder if it exists
import * as fs from "node:fs";
import * as path from "path";
import dotenv from "dotenv";

dotenv.config();

export function deleteFolderIfExists(folderPath: string) {
  if (fs.existsSync(folderPath)) {
    fs.rmSync(folderPath, { recursive: true, force: true });
    console.log(` [x] Folder '${folderPath}' deleted successfully.`);
  } else {
    console.log(` [x] Folder '${folderPath}' does not exist.`);
  }
}

export async function listDirRecursive(
  dir: string,
  indent = 0
): Promise<string> {
  let result = "";
  const files = await fs.promises.readdir(dir);

  for (const file of files) {
    if (file === ".git") continue;

    const filePath = path.join(dir, file);
    const stats = await fs.promises.stat(filePath);
    const indentation = " ".repeat(indent * 2);

    if (stats.isDirectory()) {
      result += `\t${indentation}📁 ${file}/\n`;
      result += await listDirRecursive(filePath, indent + 1);
    } else {
      result += `\t${indentation}📄 ${file}\n`;
    }
  }

  return result;
}
