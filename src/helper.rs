use crate::parsable::doctors::HasComplexResource;

pub fn collect_free_rooms_data<T:HasComplexResource>(resource:T) -> String {
    let complex_res = resource.complex_resource();
    let mut rooms_string = String::new();

    for complex in complex_res {
        match &complex.room {
            Some(v) => {
                rooms_string.push_str(
                    &format!("[{}] \n", v.availability_date.format("%d.%m.%Y").to_string())
                )
            },
            None => {
                continue;
            }
        }
    }

    if rooms_string.len() == 0 {
        "Нет записей.\n".to_string()
    } else {
        rooms_string += "\n";
        rooms_string
    }
}