use axum::{
    extract::Path,
    http::StatusCode,
    response,
    response::IntoResponse,
    routing::{delete, get, put},
    Form, Router,
};

use std::error::Error;
use std::fs::File;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct Order {
    name: String,
    monday: i32,
    tuesday: i32,
    wednesday: i32,
    thursday: i32,
    friday: i32,
    saturday: i32,
    sunday: i32,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/order/delete/:row_name", delete(delete_items))
        .route("/order/edit/:row_name", get(edit_item))
        .route("/order/:row_name", get(order_get))
        .route("/order/:row_name", put(order_put))
        .route("/order/addRow/:row_name", get(order_add_row));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn write_to_csv(orders: Vec<Order>) -> Result<(), Box<dyn Error>> {
    let file = File::create("order.csv")?;
    let mut writer = csv::Writer::from_writer(file);

    for order in orders {
        writer.serialize(order)?;
    }
    writer.flush()?;

    Ok(())
}

fn read_from_csv() -> Result<Vec<Order>, Box<dyn Error>> {
    let file = File::open("order.csv")?;
    let mut reader = csv::Reader::from_reader(file);

    let mut orders: Vec<Order> = Vec::new();

    for results in reader.deserialize() {
        let order: Order = results?;
        orders.push(order);
    }

    Ok(orders)
}

async fn index() -> impl IntoResponse {
    let orders = read_from_csv().unwrap();
    let html_start: String = String::from(
        "<html>
        <head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>Document</title>
    <script src=\"https://unpkg.com/htmx.org@1.9.10\" integrity=\"sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC\" crossorigin=\"anonymous\"></script>
            </head>
            <body>
                <table>
                    <thead>
                        <tr>
                            <th></th>
                            <th>Monday</th>
                            <th>Tuesday</th>
                            <th>Wednesday</th>
                            <th>Thursday</th>
                            <th>Friday</th>
                            <th>Saturday</th>
                            <th>Sunday</th>
                        </tr>
                    </thead>
                    <tbody hx-target=\"closest tr\" hx-swap=\"outerHTML\">
    "
    );
    let html_end: String = String::from("</tbody></body></html>");

    // let index = fs::read_to_string("client/index.html").unwrap();
    let mut index = String::new();
    index.push_str(html_start.as_str());
    index.push_str(create_table_rows(orders).as_str());
    index.push_str(html_end.as_str());

    response::Html::from(index)
}

fn create_table_rows(orders: Vec<Order>) -> String {
    let mut rows = String::new();

    for order in orders {
        rows.push_str(format_order(order).as_str());
    }
    rows
}

async fn delete_items(Path(row_name): Path<String>) -> impl IntoResponse {
    let mut orders = read_from_csv().unwrap();
    if let Some(index) = orders.iter().position(|order| order.name == row_name) {
        orders.remove(index);
        let _ = write_to_csv(orders);
    };
    StatusCode::OK
}

async fn edit_item(Path(row_name): Path<String>) -> impl IntoResponse {
    let orders = read_from_csv().unwrap();
    if let Some(index) = orders.iter().position(|order| order.name == row_name) {
        let order = orders.get(index).unwrap();
        let order_edit_html = format!(
            "
        <tr hx-trigger=\"cancel\" hx-get=\"/order/{}\">
            <td><input name=\"name\" value=\"{}\"</td>
            <td><input name=\"monday\" value=\"{}\"</td>
            <td><input name=\"tuesday\" value=\"{}\"</td>
            <td><input name=\"wednesday\" value=\"{}\"</td>
            <td><input name=\"thursday\" value=\"{}\"</td>
            <td><input name=\"friday\" value=\"{}\"</td>
            <td><input name=\"saturday\" value=\"{}\"</td>
            <td><input name=\"sunday\" value=\"{}\"</td>
            <td>
                <button hx-get=\"/order/{}\">Cancel</button>
            </td>
            <td>
                <button hx-put=\"/order/{}\" hx-include=\"closest tr\">Save</button>
            </td>
            ",
            order.name,
            order.name,
            order.monday,
            order.tuesday,
            order.wednesday,
            order.thursday,
            order.friday,
            order.saturday,
            order.sunday,
            order.name,
            order.name,
        );

        return response::Html::from(order_edit_html);
    }

    response::Html::from(String::from("<p>IDK0</p>"))
}

async fn order_get(Path(row_name): Path<String>) -> impl IntoResponse {
    let orders = read_from_csv().unwrap();
    if let Some(index) = orders.iter().position(|order| order.name == row_name) {
        let order = orders.get(index).unwrap();
        let order_html = format_order(order.clone());
        return response::Html::from(order_html);
    }
    response::Html::from(String::from("<p>IDK1</p>"))
}

async fn order_put(Path(row_name): Path<String>, form_order: Form<Order>) -> impl IntoResponse {
    let mut orders = read_from_csv().unwrap();
    if let Some(index) = orders.iter().position(|order| order.name == row_name) {
        let order = orders.get_mut(index).unwrap();
        *order = form_order.0;

        let order_html = format_order(order.clone());
        let _ = write_to_csv(orders);
        return response::Html::from(order_html);
    }
    response::Html::from(String::from("<p>IDK2</p>"))
}

async fn order_add_row(Path(row_name): Path<String>) -> impl IntoResponse {
    let placeholder_row = Order {
        name: String::from("Placeholder"),
        monday: 0,
        tuesday: 0,
        wednesday: 0,
        thursday: 0,
        friday: 0,
        saturday: 0,
        sunday: 0,
    };
    let mut orders = read_from_csv().unwrap();
    if let Some(index) = orders.iter().position(|order| order.name == row_name) {
        let order_html = format_order(placeholder_row.clone());
        orders.insert(index, placeholder_row);
        let _ = write_to_csv(orders);
        return response::Html::from(order_html);
    }
    response::Html::from(String::from("<p>IDK3</p>"))
}

fn format_order(order: Order) -> String {
    format!(
        "
<tr>
    <th>{}</th>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
        <td>
        <button class=\"btn btn-danger\" hx-delete=\"order/delete/{}\">
            Delete
        </button>
        </td>
        <td>
            <button hx-get=\"/order/edit/{}\">
            Edit
        </td>
        <td>
            <button hx-get=\"/order/addRow/{}\" hx-swap=\"afterend\">
            Add row below
        </td>
</tr>",
        order.name,
        order.monday,
        order.tuesday,
        order.wednesday,
        order.thursday,
        order.friday,
        order.saturday,
        order.sunday,
        order.name,
        order.name,
        order.name,
    )
}
