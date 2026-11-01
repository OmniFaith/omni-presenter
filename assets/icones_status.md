# Status dos Ícones do Omni Presenter

## Ícones Personalizados Ativos na Barra Lateral de Navegação

Os 3 ícones da barra lateral foram configurados com os SVGs solicitados, convertidos e normalizados perfeitamente na fonte `custom-icons.ttf`:

| Aba / Ação | Ícone SVG Utilizado | Posição / Arquivo | Descrição |
| :--- | :--- | :--- | :--- |
| **Letras / Apresentação** | [`music-note-2.svg`](file:///home/lewis/Projetos/omni-presenter/assets/icons/music-note-2.svg) | [src/ui/components/icon_sidebar.rs](file:///home/lewis/Projetos/omni-presenter/src/ui/components/icon_sidebar.rs) | Nota musical dupla na barra lateral |
| **Bíblia** | [`christian-bible.svg`](file:///home/lewis/Projetos/omni-presenter/assets/icons/christian-bible.svg) | [src/ui/components/icon_sidebar.rs](file:///home/lewis/Projetos/omni-presenter/src/ui/components/icon_sidebar.rs) | Bíblia com cruz na capa (sem nós fora do viewBox) |
| **Chat de Equipe** | [`chat-circle-2.svg`](file:///home/lewis/Projetos/omni-presenter/assets/icons/chat-circle-2.svg) | [src/ui/components/icon_sidebar.rs](file:///home/lewis/Projetos/omni-presenter/src/ui/components/icon_sidebar.rs) | Balão de diálogo arredondado com reticências |
| **Edição (`PENCIL_SIMPLE`)** | [`edit-square.svg`](file:///home/lewis/Projetos/omni-presenter/assets/icons/edit-square.svg) | Toolbars, menus de contexto e botões de editar | Quadrado de edição com caneta integrado via `custom-icons.ttf` |
| **Adicionar ao Repertório (`LIST_PLUS`)** | [`addlist.svg`](file:///home/lewis/Projetos/omni-presenter/assets/icons/addlist.svg) | [src/ui/pages/bible/selection.rs](file:///home/lewis/Projetos/omni-presenter/src/ui/pages/bible/selection.rs), seleção e design | Lista com sinal de adição para adicionar seleções ao repertório |

> **Nota de integridade visual:** Todos os ícones customizados (`music-note-2`, `christian-bible`, `chat-circle-2`, `edit-square` e `addlist`) foram normalizados em curvas vetoriais fechadas dentro do viewBox (0..1000) e embutidos diretamente na fonte `custom-icons.ttf`. O restante dos ícones continua com fallback limpo para `egui_phosphor`, garantindo rendering sem artefatos ou linhas diagonais em qualquer resolução.

## Sincronização Completa com a Interface Web (Controle e Chat)

A interface web (tanto o **Controle Web** quanto o **Chat de Equipe**) agora utiliza **exatamente a mesma fonte vetorial e os mesmos glifos do aplicativo nativo**:

1. **Fonte Vetorial Unificada no Servidor Web:**
   - A fonte `assets/fonts/custom-icons.ttf` foi compilada contendo todos os glifos nativos (Phosphor) integrados aos 5 ícones customizados (`music-note-2`, `christian-bible`, `chat-circle-2`, `addlist` e `edit-square`).
   - Servida nas rotas `/api/control/icon-font` em [src/services/control/http.rs](file:///home/lewis/Projetos/omni-presenter/src/services/control/http.rs) e `/api/chat/icon-font` em [src/services/chat/http.rs](file:///home/lewis/Projetos/omni-presenter/src/services/chat/http.rs).
2. **Renderização Direta por Glifos Nativos:**
   - Em [assets/web/control/control-icons.js](file:///home/lewis/Projetos/omni-presenter/assets/web/control/control-icons.js) e [assets/web/chat/chat-icons.js](file:///home/lewis/Projetos/omni-presenter/assets/web/chat/chat-icons.js), a função `icon()` gera diretamente elementos `<span class="native-icon">` mapeados para os mesmos caracteres Unicode da aplicação desktop.
   - Tanto [assets/web/control/control-shell.css](file:///home/lewis/Projetos/omni-presenter/assets/web/control/control-shell.css) quanto [assets/web/chat/chat.css](file:///home/lewis/Projetos/omni-presenter/assets/web/chat/chat.css) carregam a fonte `NativeIcons`, garantindo fidelidade pixel-a-pixel entre o desktop e a web.
3. **Botão de Adicionar da Bíblia:**
   - O botão `#bible-add` em [assets/web/control/control.html](file:///home/lewis/Projetos/omni-presenter/assets/web/control/control.html) utiliza `data-icon="addlist"`, renderizando o mesmo ícone `addlist.svg`.


