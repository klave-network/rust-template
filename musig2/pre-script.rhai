let project_name = variable::get("project-name");
let component = project_name;

// Removing numeric characters from component name
let digits_to_remove = [1, 2, 3, 4, 5, 6, 7, 6, 9];
for item in digits_to_remove {
    if project_name.contains(item.to_string()){
        component.replace(item.to_string(), "");
    }
}

// ANSI codes for formatting
let bold_start = "\x1b[1m";      // Start bold

let component_kebab_name = to_kebab_case(component);
print(`🔧   ${bold_start}component-name: ${component_kebab_name} ...`);

// Set component_name variable
variable::set("component_name", component_kebab_name);
