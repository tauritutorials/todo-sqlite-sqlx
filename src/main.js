const { invoke } = window.__TAURI__.core;

async function addTodo(description) {
    return await invoke("add_todo", {
        description,
    });
}

async function getTodos() {
    return await invoke("get_todos");
}

async function updateTodo(todo) {
    return await invoke("update_todo", { todo });
}

async function deleteTodo(id) {
    return await invoke("delete_todo", { id });
}

async function buildTodoList() {
    let todos = await getTodos();
    let tasksContainer = document.querySelector("#tasks");
    tasksContainer.innerHTML = "";

    todos.forEach((todo) => {
        let div = document.createElement("div");
        div.classList.add("todo-wrapper");

        div.innerHTML = `
            <label>
                <input type="checkbox" class="todo-item" data-id="${todo.id}" data-description="${todo.description}" ${todo.status === "Complete" ? "checked='checked'" : ""}>
                <span>${todo.description}</span>
            </label>

            <button class="delete" data-id="${todo.id}">
                delete
            </button>
        `;

        tasksContainer.appendChild(div);
    });

    document.querySelectorAll(".todo-item").forEach((el) => {
        el.addEventListener("input", (input) => {
            let data = input.target.dataset;
            updateTodo({
                id: parseInt(data.id),
                description: data.description,
                status: input.target.checked ? "Complete" : "Incomplete",
            });
        });
    });

    document.querySelectorAll(".delete").forEach((el) => {
        el.addEventListener("click", async (event) => {
            let id = parseInt(event.target.dataset.id);

            await deleteTodo(id);
            await buildTodoList();
        });
    });
}

window.addEventListener("DOMContentLoaded", () => {
    buildTodoList();

    document.querySelector("#todo-form").addEventListener("submit", (event) => {
        event.preventDefault();

        let input = document.querySelector("#todo-input");

        addTodo(input.value).then(() => {
            buildTodoList();
        });

        input.value = "";
    });
});
