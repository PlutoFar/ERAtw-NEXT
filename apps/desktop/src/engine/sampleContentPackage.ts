import type { ContentPackage } from "../types";

export const createSampleContentPackage = (): ContentPackage => ({
  manifest: {
    schema_version: "content-package/v0",
    namespace: "sample",
    package_id: "sample.event_pack",
    version: "0.1.0",
    dependencies: [],
    conflicts: [],
  },
  locations: [
    {
      id: "sample_studio",
      name: "样例工房",
      ascii_symbol: "样",
      terrain: "interior",
    },
  ],
  text_maps: [],
  characters: [
    {
      id: "sample_guest",
      display_name: "样例来客",
      location_id: "sample_studio",
      state: {
        energy: 70,
        mood: 8,
      },
    },
  ],
  relationships: [
    {
      source_character_id: "player",
      target_character_id: "sample_guest",
      affinity: 1,
      trust: 0,
    },
  ],
  resources: [
    {
      resource_id: "sample.event_pack.guest.smile",
      source_path: "assets/sample/guest-smile.webp",
      media_type: "image",
      license: "project-demo",
      author: "ERAtw-NEXT",
      usage: ["portrait", "dialogue"],
      character_bindings: ["sample_guest"],
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
          speaker_id: "sample_guest",
          text: "我是随内容包新增的角色。这段对话没有经过旧 ERB 执行。",
          resource_refs: ["sample.event_pack.guest.smile"],
          choices: [
            {
              id: "acknowledge",
              label: "记录下来",
              next_node_id: null,
              conditions: [],
              effects: [
                {
                  type: "adjust_relationship",
                  source_character_id: "player",
                  target_character_id: "sample_guest",
                  affinity_delta: 1,
                  trust_delta: 1,
                },
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
