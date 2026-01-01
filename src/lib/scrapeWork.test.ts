import { describe, it, expect } from "vitest";
import { getCreator, getVoiceActors, getMusics } from "./scrapeWork";

describe("scrapeWork", () => {
    describe("getCreator", () => {
        it("should extract creators from anchor tags", () => {
            const container = document.createElement("div");
            container.innerHTML = `
        <a href="creater.php?creater=101">Creator A</a>
        <a href="creater.php?creater=202">Creator B</a>
      `;
            const creators = getCreator(container);
            expect(creators).toHaveLength(2);
            expect(creators[0]).toEqual({ id: 101, name: "Creator A" });
            expect(creators[1]).toEqual({ id: 202, name: "Creator B" });
        });

        it("should handle links without ID gracefully", () => {
            const container = document.createElement("div");
            container.innerHTML = `<a href="other.php">Unknown</a>`;
            const creators = getCreator(container);
            expect(creators).toHaveLength(1);
            expect(creators[0].id).toBe(0);
            expect(creators[0].name).toBe("Unknown");
        });
    });

    describe("getVoiceActors", () => {
        it("should extract voice actors with roles and importance", () => {
            // Voice Actor structure: <a>Name</a> ... <span style="...">Role</span>
            const container = document.createElement("div");

            // 1. Black style (importance 1)
            // 2. Bold style (importance 0 - main)
            // 3. Other style (importance 2 - mob)
            container.innerHTML = `
        <a href="creater.php?creater=10">Actor A</a>
        <span style="color:black">Role A</span>
        
        <a href="creater.php?creater=20">Actor B</a>
        <span style="font-weight:bold">Role B</span>
        
        <a href="creater.php?creater=30">Actor C</a>
        <span style="color:gray">Role C</span>
      `;
            const actors = getVoiceActors(container);

            expect(actors).toHaveLength(3);

            expect(actors[0].id).toBe(10);
            expect(actors[0].name).toBe("Actor A");
            expect(actors[0].role).toBe("Role A");
            expect(actors[0].importance).toBe(1); // black -> 1

            expect(actors[1].id).toBe(20);
            expect(actors[1].name).toBe("Actor B");
            expect(actors[1].role).toBe("Role B");
            expect(actors[1].importance).toBe(0); // bold -> 0

            expect(actors[2].id).toBe(30);
            expect(actors[2].name).toBe("Actor C");
            expect(actors[2].role).toBe("Role C");
            expect(actors[2].importance).toBe(2); // other -> 2
        });
    });

    describe("getMusics", () => {
        it("should extract music titles from table cells", () => {
            const row = document.createElement("tr");
            const cell1 = document.createElement("td");
            cell1.innerHTML = `<a href="music.php?music=1">Music 1</a>`;
            const cell2 = document.createElement("td");
            cell2.innerHTML = `<a href="music.php?music=2">Music 2</a>`;
            const cell3 = document.createElement("td");
            cell3.innerHTML = `Just Text`; // Ignored

            // HTMLCollectionを作成するのは難しいので、getElementsByTagNameの結果を使う
            const table = document.createElement("table");
            row.appendChild(cell1);
            row.appendChild(cell2);
            row.appendChild(cell3);
            table.appendChild(row);

            const cells = row.getElementsByTagName("td");
            const musics = getMusics(cells);

            expect(musics).toHaveLength(2);
            expect(musics[0]).toBe("Music 1");
            expect(musics[1]).toBe("Music 2");
        });
    });
});
