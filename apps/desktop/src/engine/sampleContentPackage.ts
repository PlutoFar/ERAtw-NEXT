import type { ContentPackage } from "../types";

export const createSampleContentPackage = (): ContentPackage => ({
  manifest: {
    schema_version: "content-package/v0",
    namespace: "sample",
    package_id: "sample.event_pack",
    version: "0.1.0",
    dependencies: [],
  },
  resources: [
    {
      resource_id: "sample.event_pack.heroine.smile",
      source_path: "assets/sample/heroine-smile.webp",
      media_type: "image",
      license: "project-demo",
      author: "ERAtw-NEXT",
      usage: ["portrait", "dialogue"],
      character_bindings: ["demo_heroine"],
      tags: ["smile", "sample"],
      sha256: null,
    },
  ],
  dialogue_scenes: [
    {
      id: "sample_event_dialogue",
      entry_node_id: "sample_event_entry",
      nodes: [
        {
          id: "sample_event_entry",
          speaker_id: "demo_heroine",
          text: "这是从内容包安装进来的事件对话。它没有经过旧 ERB 执行。",
          resource_refs: ["sample.event_pack.heroine.smile"],
          choices: [
            {
              id: "acknowledge",
              label: "记录下来",
              next_node_id: null,
              conditions: [],
              effects: [
                {
                  type: "add_log",
                  message: "内容包示例对话已确认。",
                },
              ],
            },
          ],
        },
      ],
    },
  ],
  scheduled_events: [
    {
      id: "sample_content_dialogue_at_0820",
      due: { day: 1, hour: 8, minute: 20 },
      priority: 5,
      repeat: null,
      conditions: [],
      kind: {
        type: "start_dialogue",
        scene_id: "sample_event_dialogue",
      },
    },
  ],
});
