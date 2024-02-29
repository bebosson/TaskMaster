use super::{
    conf::Taskmaster,
    task::Task,
    proc::Proc
};

impl Taskmaster {
    pub fn get_all_task_html(& mut self) -> String{
        let mut big_string: String = "\t\t\t<tr>\n".to_string();
        big_string = format!("{}\t\t\t\t<th class=\"state\" >State</th>\n",big_string);
        big_string = format!("{}\t\t\t\t<th class=\"process\">Process</th>\n",big_string);
        big_string = format!("{}\t\t\t\t<th class= \"desc\">Description</th>\n",big_string);
        big_string = format!("{}\t\t\t\t<th class=\"action\">Action</th>\n",big_string);
        big_string = format!("{}\t\t\t</tr>\n",big_string);
        // self.print_all_stats(); // a supprimer
        for i in &self.task_lst{
            big_string = format!("{}{}",big_string,i.get_all_proc_html());
        }
        big_string = format!("\t\t<table cellspacing=\"0\">\n{}\n\t\t</table>\n",big_string);
        big_string = format!("\t<body>\n{}\n\t</body>\n",big_string);

        // println!("{}", big_string);
        big_string
    }
}

impl Task{
    pub fn get_all_proc_html(&self) -> String {
        let mut tmp_string = "".to_string();
        for i in &self.process_lst {
            tmp_string = format!("{}{}", tmp_string, i.get_proc_html());
        }
        tmp_string
    }
}

impl Proc{
    pub fn get_proc_html(&self) -> String {
        let mut ret_string;
        let state: String;
        let name: String;
        let class_action: String;
        let description: String;
        
        name = format!("{}{:?}{}", "\t\t\t\t<td>", self.name.as_ref().unwrap(), "</td>\n");
        state = format!("{}{:?}{}","\t\t\t\t<td>",self.state, "</td>\n");
        description = format!("{}{:?}{}","\t\t\t\t<td>",self.description.as_ref().unwrap(), "</td>\n");
        class_action = "\t\t\t\t<td class = \"action\">\n".to_string();
        // action_0 = self.add_button_action(self.get_action_string());
        // action_1 = self.add_button_action("clearlog".to_string());
        // action_2 = self.add_button_action("tail -f stdout".to_string());
        // action_3 = self.add_button_action("tail -f stderr".to_string());
        // button_action = format!("{}{}{}{}{}", "\t\t\t\t<td class=\"action\">", "<button type=\"button\"",action,"</button>".to_string(), "</td>\n");
        // button_start = format();
        // println!("{}", state);
        // println!("{}", name);
        // ret_string = format!("\t\t\t<tr>\n{}{}{}{}\t\t\t</tr>\n", state, name, description, button_action);
        ret_string = format!("\t\t\t<tr>\n{}{}{}{}{}\t\t\t</tr>\n", state, name, description, class_action, self.add_all_button());
        ret_string = format!("{}\t\t\t\t</td>", ret_string);
        ret_string
    }

    pub fn add_all_button(&self) -> String {
        let mut all_action: String = String::new();
        for i in &self.get_all_action_string()
        {
            all_action = format!("{}{}", self.add_button_action(i), all_action);
        }
        all_action = format!("{}{}", all_action, "\t\t\t\t</ul>\n");
        all_action
    }

    pub fn add_button_action(&self, string_action: &String) -> String {
        let mut all_action: String;

        // all_action = "\t\t\t\t<td class=\"action\">\n".to_string();
        all_action = "\t\t\t\t\t<ul>\n".to_string();
        all_action = format!("{}\t\t\t\t\t\t<li>\n", all_action);
        // println!("{:?}",self.name.as_ref().expect("option name"));
        // panic!("wtf");
        all_action = format!("{}\t\t\t\t\t\t\t<a href=\"index.html?process_name={}&action={}\" name={:?}> {}</a>\n", all_action, self.name.as_ref().expect("get_all_button"),string_action,self.name.as_ref().expect("get_all_button"), string_action);
        // println!("{:?}", all_action);
        // panic!("debug");
        all_action = format!("{}\t\t\t\t\t\t</li>\n", all_action);

        all_action
    }
}

// fn normal_response(taskm: &mut Taskmaster) -> String {
//     let str_in = taskm.get_all_task_html();
//     let mut response =
//         "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".to_string();
//     response = format!("{}{}", response, include_str!("index_cpy.html"));
//     response = format!("{}{}</html>", response, str_in);
//     response
// }

// fn test_response(taskm: &mut Taskmaster, request_line: &String) -> String {
//     let normal_request = request_line.eq(&"GET / HTTP/1.1".to_string());
//     match normal_request {
//         true => (),
//         false => request_response(taskm, request_line),
//     }
//     taskm.print_all_stats();
//     normal_response(taskm)
// }

// fn type_of_request(request: String, taskm: &mut Taskmaster) -> String {
//     if request.find("GET").is_some() == true
//     // check if html
//     {
//         test_response(taskm, &request)
//     } else {
//         // come from taskctl
//         correct_name_process(&request, taskm)
//     }
// }