import { copy, walk } from "@std/fs";
import { join, parse, relative } from "@std/path";

const cwd = Deno.cwd();
const baseOutDir = join(cwd, "dist");

// 1. Process manifest
const manifestText = await Deno.readTextFile(join(cwd, "manifest.json"));
const manifest = JSON.parse(manifestText);

const basePath: string | undefined = manifest.basePath;
const outDir = basePath ? join(baseOutDir, basePath) : baseOutDir;

// 2. Process static resources
const resourcesDir = join(cwd, "resources");
const outResourcesDir = join(outDir, "resources");

const resourceMap = new Map<string, string>();

// 2.1. Copy directory
await copy(resourcesDir, outResourcesDir, {
    overwrite: true,
    preserveTimestamps: true,
});

// 2.2. Process resource mappings
type ResourceMappings = Record<string, string | Record<string, string>>;

try {
    const resourceMappingsText = await Deno.readTextFile(
        join(resourcesDir, "map.json"),
    );
    const resourceMappings: ResourceMappings = JSON.parse(resourceMappingsText);

    //TODO: Duplicate checking
    for (const [keyOrUrl, value] of Object.entries(resourceMappings)) {
        if (typeof value === "string") {
            resourceMap.set(keyOrUrl, value);
            continue;
        }

        for (const [key, rest] of Object.entries(value)) {
            const joined = join(keyOrUrl, rest);
            resourceMap.set(key, joined);
        }
    }
} catch (e) {
    //TODO: log message/error
    throw e;
}

// 2.3. Process other resources
for await (
    const entry of walk(outResourcesDir, { includeDirs: false })
) {
    if (entry.name === "map.json") {
        continue;
    }

    const relativePath = relative(outDir, entry.path);
    const path = parse(entry.path);

    if (!resourceMap.has(path.name)) {
        resourceMap.set(path.name, relativePath);
    } else if (!resourceMap.has(entry.name)) {
        // Remove the existing resource, add it back with the file extension,
        // and add the new resource
        resourceMap.delete(path.name);
        //TODO: Transform resource name as specified above
    } else {
        //TODO: Duplicate key error
    }
}
